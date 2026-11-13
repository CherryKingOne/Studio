use std::collections::HashMap;
use std::path::PathBuf;

use bytes::Bytes;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::audio::types::{
    LoudnessOptions, MergeOptions, MergePayload, MergeRequest, OutputOptions, PauseOptions,
};
use crate::delivery::files::repo::NewFile;
use crate::delivery::files::service as files_service;
use crate::library::books::dto::BookSettings;
use crate::library::books::repo as books_repo;
use crate::library::books::service as books_service;
use crate::library::chapters::repo as chapters_repo;
use crate::library::chapters::service as chapters_service;
use crate::library::segments::dto::SegmentListQuery;
use crate::library::segments::model::Segment;
use crate::library::segments::repo as segments_repo;
use crate::runtime::error::{AppError, FieldError};
use crate::runtime::state::AppState;
use crate::shared::ffmpeg::{self, MergeSpec, SilenceSpec};
use crate::shared::temp::TempDir;
use crate::shared::text::sanitize_filename;
use crate::task_center::model::Job;
use crate::task_center::queue as jobs_queue;
use crate::task_center::repo as jobs_repo;
use crate::task_center::repo::NewJob;

pub async fn create_merge_job(
    state: &AppState,
    chapter_id: u64,
    request: MergeRequest,
    user_id: u64,
) -> Result<u64, AppError> {
    let chapter = chapters_service::get_model(state, chapter_id).await?;
    let book = books_service::get_model(state, chapter.book_id).await?;
    if let Some(existing_job_id) =
        jobs_repo::active_chapter_job_id(&state.db, chapter_id, "chapter_merge").await?
    {
        return Err(AppError::JobAlreadyActive { existing_job_id });
    }

    let segments = segments_repo::list_by_chapter(
        &state.db,
        chapter_id,
        &SegmentListQuery {
            seg_type: None,
            status: None,
            speaker_id: None,
        },
    )
    .await?;
    let done_segments = segments
        .iter()
        .filter(|segment| segment.status == "done")
        .count();
    if done_segments == 0 {
        return Err(AppError::InvalidState("章节没有已合成的段落".to_string()));
    }
    let incomplete = segments.iter().any(|segment| segment.status != "done");
    if incomplete && !request.skip_failed {
        return Err(AppError::InvalidState(
            "存在未完成段落，请先完成合成或设置 skip_failed=true".to_string(),
        ));
    }

    let book_settings: BookSettings =
        serde_json::from_value(book.settings.0.clone()).unwrap_or_default();
    let options = merge_options(&book_settings, request)?;

    let payload = MergePayload {
        chapter_id,
        options,
    };
    let job = NewJob {
        job_type: "chapter_merge".to_string(),
        priority: 4,
        book_id: Some(book.id),
        chapter_id: Some(chapter.id),
        created_by: Some(user_id),
        payload: serde_json::to_value(&payload)?,
        total_items: done_segments as u32,
        max_attempts: 2,
    };
    jobs_queue::enqueue(&state.db, &job, &[]).await
}

pub async fn run_merge(state: &AppState, job: &Job) -> Result<Value, String> {
    let payload: MergePayload =
        serde_json::from_value(job.payload.0.clone()).map_err(|error| error.to_string())?;
    let chapter = chapters_service::get_model(state, payload.chapter_id)
        .await
        .map_err(|error| error.to_string())?;
    let book = books_service::get_model(state, chapter.book_id)
        .await
        .map_err(|error| error.to_string())?;
    let segments: Vec<Segment> = segments_repo::list_by_chapter(
        &state.db,
        chapter.id,
        &SegmentListQuery {
            seg_type: None,
            status: None,
            speaker_id: None,
        },
    )
    .await
    .map_err(|error| error.to_string())?
    .into_iter()
    .filter(|segment| segment.status == "done" && segment.audio_file_id.is_some())
    .collect();
    if segments.is_empty() {
        return Err("没有可拼接的段落音频".to_string());
    }

    let tmp = TempDir::create(&state.config.audio.tmp_dir, &format!("merge-{}", job.id))
        .await
        .map_err(|error| error.to_string())?;
    let options = &payload.options;

    let mut segment_paths = Vec::with_capacity(segments.len());
    for (index, segment) in segments.iter().enumerate() {
        if jobs_repo::is_cancel_requested(&state.db, job.id)
            .await
            .map_err(|error| error.to_string())?
        {
            return Err("任务已取消".to_string());
        }
        let file_id = segment
            .audio_file_id
            .ok_or_else(|| format!("段落 {} 缺少音频", segment.id))?;
        let file = files_service::get(state, file_id)
            .await
            .map_err(|error| error.to_string())?;
        let bytes = state
            .storage
            .get(&file.object_key)
            .await
            .map_err(|error| error.to_string())?;
        let path = tmp.path().join(format!("{:05}.mp3", segment.idx));
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(|error| error.to_string())?;
        segment_paths.push((path, segment.clone()));

        let progress = 5 + ((index + 1) * 30 / segments.len()) as u8;
        let _ = jobs_repo::update_progress(
            &state.db,
            job.id,
            progress,
            0,
            0,
            &format!("下载段落音频 {}/{}", index + 1, segments.len()),
        )
        .await;
    }

    let mut silence_cache: HashMap<u32, PathBuf> = HashMap::new();
    for duration in [
        options.pause_ms.narration,
        options.pause_ms.dialogue,
        options.pause_ms.chapter_end,
    ] {
        if duration == 0 || silence_cache.contains_key(&duration) {
            continue;
        }
        let path = tmp.path().join(format!("silence-{duration}.mp3"));
        ffmpeg::generate_silence(&SilenceSpec {
            ffmpeg_path: &state.config.audio.ffmpeg_path,
            output: &path,
            duration_ms: duration,
            sample_rate: options.output.sample_rate,
            channels: options.output.channels,
            bitrate_kbps: options.output.bitrate_kbps,
        })
        .await
        .map_err(|error| error.to_string())?;
        silence_cache.insert(duration, path);
    }

    let concat_path = tmp.path().join("concat.txt");
    let mut concat = String::new();
    for (path, segment) in &segment_paths {
        concat.push_str(&format!("file '{}'\n", path.display()));
        let pause = if segment.seg_type == "dialogue" {
            options.pause_ms.dialogue
        } else {
            options.pause_ms.narration
        };
        if let Some(silence) = silence_cache.get(&pause) {
            concat.push_str(&format!("file '{}'\n", silence.display()));
        }
    }
    if let Some(silence) = silence_cache.get(&options.pause_ms.chapter_end) {
        concat.push_str(&format!("file '{}'\n", silence.display()));
    }
    tokio::fs::write(&concat_path, concat)
        .await
        .map_err(|error| error.to_string())?;

    let _ = jobs_repo::update_progress(&state.db, job.id, 40, 0, 0, "FFmpeg 拼接与响度处理").await;
    let output_path = tmp.path().join("chapter.mp3");
    ffmpeg::merge_with_loudnorm(&MergeSpec {
        ffmpeg_path: &state.config.audio.ffmpeg_path,
        concat_file: &concat_path,
        output: &output_path,
        sample_rate: options.output.sample_rate,
        channels: options.output.channels,
        bitrate_kbps: options.output.bitrate_kbps,
        loudness_i: options.loudness.i,
        loudness_tp: options.loudness.tp,
        loudness_lra: options.loudness.lra,
        title: &chapter.title,
        artist: book.author.as_deref().unwrap_or("未知作者"),
        album: &book.title,
        track: chapter.idx,
    })
    .await
    .map_err(|error| error.to_string())?;

    let bytes = tokio::fs::read(&output_path)
        .await
        .map_err(|error| error.to_string())?;
    let duration_ms = ffmpeg::probe_duration_ms(
        &state.config.audio.ffprobe_path,
        &state.config.audio.tmp_dir,
        &bytes,
    )
    .await
    .unwrap_or(0);

    let rev = chapter.audio_rev + 1;
    let object_key = format!("audio/chapters/{}/{}-{}.mp3", book.id, chapter.id, rev);
    state
        .storage
        .put(&object_key, Bytes::from(bytes.clone()), "audio/mpeg")
        .await
        .map_err(|error| error.to_string())?;
    let sha256 = hex::encode(Sha256::digest(&bytes));
    let file_id = files_service::register(
        state,
        NewFile {
            bucket: state.storage.bucket().to_string(),
            object_key,
            file_name: format!(
                "{:03}-{}.mp3",
                chapter.idx,
                sanitize_filename(&chapter.title)
            ),
            mime: "audio/mpeg".to_string(),
            size_bytes: bytes.len() as u64,
            sha256,
            kind: "chapter_audio".to_string(),
            duration_ms: Some(duration_ms),
        },
    )
    .await
    .map_err(|error| error.to_string())?;

    chapters_repo::update_audio(&state.db, chapter.id, file_id, duration_ms, rev)
        .await
        .map_err(|error| error.to_string())?;
    let _ = books_repo::update_total_duration(&state.db, book.id).await;
    let _ =
        jobs_repo::update_progress(&state.db, job.id, 100, segments.len() as u32, 0, "拼接完成")
            .await;

    Ok(json!({
        "chapter_id": chapter.id,
        "file_id": file_id,
        "duration_ms": duration_ms,
        "skipped_segments": segments.len(),
    }))
}

pub async fn chapter_audio(state: &AppState, chapter_id: u64) -> Result<(Bytes, String), AppError> {
    let chapter = chapters_service::get_model(state, chapter_id).await?;
    let file_id = chapter.audio_file_id.ok_or(AppError::NotFound {
        resource: "章节音频",
    })?;
    let file = files_service::get(state, file_id).await?;
    let bytes = state.storage.get(&file.object_key).await?;
    Ok((
        bytes,
        format!(
            "{:03}-{}.mp3",
            chapter.idx,
            sanitize_filename(&chapter.title)
        ),
    ))
}

pub async fn delete_chapter_audio(state: &AppState, chapter_id: u64) -> Result<(), AppError> {
    let chapter = chapters_service::get_model(state, chapter_id).await?;
    if chapter.audio_file_id.is_none() {
        return Err(AppError::NotFound {
            resource: "章节音频",
        });
    }
    chapters_service::clear_audio(state, chapter_id).await?;
    let (total, done, _failed) = segments_repo::chapter_counts(&state.db, chapter_id).await?;
    let status = if done == total {
        "tts_done"
    } else {
        "tts_partial"
    };
    chapters_service::update_status(state, chapter_id, status).await?;
    Ok(())
}

pub async fn segment_audio(state: &AppState, segment_id: u64) -> Result<(Bytes, String), AppError> {
    let segment = segments_repo::find_by_id(&state.db, segment_id)
        .await?
        .ok_or(AppError::NotFound { resource: "段落" })?;
    let file_id = segment.audio_file_id.ok_or(AppError::NotFound {
        resource: "段落音频",
    })?;
    let file = files_service::get(state, file_id).await?;
    let bytes = state.storage.get(&file.object_key).await?;
    Ok((bytes, format!("segment-{}.mp3", segment_id)))
}

fn merge_options(settings: &BookSettings, request: MergeRequest) -> Result<MergeOptions, AppError> {
    let pause = request.pause_ms.unwrap_or(PauseOptions {
        narration: settings.pause_ms.narration,
        dialogue: settings.pause_ms.dialogue,
        chapter_end: settings.pause_ms.chapter_end,
    });
    let loudness = request.loudness.unwrap_or(LoudnessOptions {
        i: settings.loudness.i,
        tp: settings.loudness.tp,
        lra: settings.loudness.lra,
    });
    let output = request.output.unwrap_or(OutputOptions {
        sample_rate: settings.output.sample_rate,
        channels: settings.output.channels,
        bitrate_kbps: settings.output.bitrate_kbps,
    });

    if pause.narration > 5000 || pause.dialogue > 5000 || pause.chapter_end > 10_000 {
        return Err(validation("pause_ms", "停顿时间超出范围"));
    }
    if !(-40.0..=-5.0).contains(&loudness.i)
        || !(-9.0..=0.0).contains(&loudness.tp)
        || !(1.0..=50.0).contains(&loudness.lra)
    {
        return Err(validation("loudness", "响度参数超出范围"));
    }
    if !matches!(output.sample_rate, 24_000 | 44_100 | 48_000) {
        return Err(validation("output.sample_rate", "仅支持 24000/44100/48000"));
    }
    if !matches!(output.channels, 1 | 2) {
        return Err(validation("output.channels", "仅支持 1 或 2"));
    }
    if !(64..=320).contains(&output.bitrate_kbps) {
        return Err(validation("output.bitrate_kbps", "必须在 64 到 320 之间"));
    }

    Ok(MergeOptions {
        skip_failed: request.skip_failed,
        pause_ms: pause,
        loudness,
        output,
    })
}

fn validation(field: &str, message: &str) -> AppError {
    AppError::Validation(vec![FieldError {
        field: field.to_string(),
        message: message.to_string(),
    }])
}
