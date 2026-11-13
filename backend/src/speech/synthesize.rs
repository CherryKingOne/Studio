use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::casting::voices::service as voices_service;
use crate::delivery::files::repo::NewFile;
use crate::delivery::files::service as files_service;
use crate::library::chapters::service as chapters_service;
use crate::library::segments::dto::SegmentListQuery;
use crate::library::segments::model::Segment;
use crate::library::segments::repo as segments_repo;
use crate::runtime::error::{AppError, FieldError};
use crate::runtime::state::AppState;
use crate::shared::ffmpeg;
use crate::speech::cache::{self, NewCacheEntry};
use crate::speech::providers::{SynthesisRequest, TtsProvider};
use crate::speech::types::{CacheParams, TtsJobResponse, TtsPayload, TtsRunResult};
use crate::task_center::model::Job;
use crate::task_center::queue as jobs_queue;
use crate::task_center::repo as jobs_repo;
use crate::task_center::repo::NewJob;

pub async fn create_chapter_job(
    state: &AppState,
    chapter_id: u64,
    mode: &str,
    force: bool,
    user_id: u64,
) -> Result<TtsJobResponse, AppError> {
    let chapter = chapters_service::get_model(state, chapter_id).await?;
    if let Some(existing_job_id) =
        jobs_repo::active_chapter_job_id(&state.db, chapter_id, "chapter_tts").await?
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

    let candidates: Vec<Segment> = match mode {
        "missing" => segments
            .into_iter()
            .filter(|segment| matches!(segment.status.as_str(), "pending" | "failed"))
            .collect(),
        "failed" => segments
            .into_iter()
            .filter(|segment| segment.status == "failed")
            .collect(),
        "all" => segments,
        _ => {
            return Err(AppError::Validation(vec![FieldError {
                field: "mode".to_string(),
                message: "只能是 missing / failed / all".to_string(),
            }]));
        }
    };

    if candidates.is_empty() {
        return Err(AppError::InvalidState("没有需要合成的段落".to_string()));
    }

    create_job_for_segments(state, chapter, candidates, force, user_id).await
}

pub async fn create_segment_job(
    state: &AppState,
    segment_id: u64,
    force: bool,
    user_id: u64,
) -> Result<TtsJobResponse, AppError> {
    let segment = segments_repo::find_by_id(&state.db, segment_id)
        .await?
        .ok_or(AppError::NotFound { resource: "段落" })?;
    let chapter = chapters_service::get_model(state, segment.chapter_id).await?;
    create_job_for_segments(state, chapter, vec![segment], force, user_id).await
}

async fn create_job_for_segments(
    state: &AppState,
    chapter: crate::library::chapters::model::Chapter,
    candidates: Vec<Segment>,
    force: bool,
    user_id: u64,
) -> Result<TtsJobResponse, AppError> {
    let resolved = voices_service::resolve_for_chapter(state, chapter.id, &candidates).await?;
    let mut engines: HashMap<String, Arc<dyn TtsProvider>> = HashMap::new();
    let mut segment_ids = Vec::new();
    let mut cached_hits = 0u64;

    for segment in &candidates {
        let Some(voice) = resolved.get(&segment.id) else {
            continue;
        };
        let Some(provider_voice_id) = voice.provider_voice_id.clone() else {
            continue;
        };
        let engine = if let Some(engine) = engines.get(&voice.provider) {
            engine.clone()
        } else {
            let engine = state.tts.provider(&voice.provider)?;
            engines.insert(voice.provider.clone(), engine.clone());
            engine
        };
        let key = cache::cache_key(
            &voice.provider,
            engine.version(),
            &provider_voice_id,
            &segment.text,
            &segment.emotion,
            segment.rate_percent,
            segment.pitch_hz,
            segment.volume_percent,
        );
        if !force && cache::get(&state.db, &key).await?.is_some() {
            cached_hits += 1;
        }
        segment_ids.push(segment.id);
    }

    if segment_ids.is_empty() {
        return Err(AppError::InvalidState(
            "没有可用音色的段落，请先登记音色".to_string(),
        ));
    }

    let payload = TtsPayload {
        chapter_id: chapter.id,
        segment_ids: segment_ids.clone(),
        force,
    };
    let job = NewJob {
        job_type: "chapter_tts".to_string(),
        priority: 5,
        book_id: Some(chapter.book_id),
        chapter_id: Some(chapter.id),
        created_by: Some(user_id),
        payload: serde_json::to_value(&payload)?,
        total_items: segment_ids.len() as u32,
        max_attempts: state.config.tts.max_attempts.max(1),
    };
    let job_id = jobs_queue::enqueue(&state.db, &job, &[]).await?;

    Ok(TtsJobResponse {
        job_id,
        total_candidates: segment_ids.len() as u64,
        cached_hits,
    })
}

pub async fn run_tts(state: &AppState, job: &Job) -> Result<TtsRunResult, String> {
    let payload: TtsPayload =
        serde_json::from_value(job.payload.0.clone()).map_err(|error| error.to_string())?;
    let chapter = chapters_service::get_model(state, payload.chapter_id)
        .await
        .map_err(|error| error.to_string())?;
    let book = crate::library::books::service::get_model(state, chapter.book_id)
        .await
        .map_err(|error| error.to_string())?;

    let segments = segments_repo::find_by_ids(&state.db, &payload.segment_ids)
        .await
        .map_err(|error| error.to_string())?;
    let total = segments.len().max(1) as u64;
    let resolved = voices_service::resolve_for_chapter(state, chapter.id, &segments)
        .await
        .map_err(|error| error.to_string())?;

    let mut engines: HashMap<String, Arc<dyn TtsProvider>> = HashMap::new();
    let mut done = 0u64;
    let mut failed = 0u64;
    let mut cached = 0u64;

    for segment in &segments {
        if jobs_repo::is_cancel_requested(&state.db, job.id)
            .await
            .map_err(|error| error.to_string())?
        {
            return Err("任务已取消".to_string());
        }

        let claimed = segments_repo::claim_for_synthesis(&state.db, segment.id)
            .await
            .map_err(|error| error.to_string())?;
        if !claimed {
            continue;
        }

        let Some(voice) = resolved.get(&segment.id) else {
            segments_repo::mark_failed(&state.db, segment.id, job.max_attempts, "未解析到音色")
                .await
                .map_err(|error| error.to_string())?;
            failed += 1;
            continue;
        };
        let Some(provider_voice_id) = voice.provider_voice_id.clone() else {
            segments_repo::mark_failed(&state.db, segment.id, job.max_attempts, "未登记音色")
                .await
                .map_err(|error| error.to_string())?;
            failed += 1;
            continue;
        };

        let engine = if let Some(engine) = engines.get(&voice.provider) {
            engine.clone()
        } else {
            let engine = state
                .tts
                .provider(&voice.provider)
                .map_err(|error| error.to_string())?;
            engines.insert(voice.provider.clone(), engine.clone());
            engine
        };

        let key = cache::cache_key(
            &voice.provider,
            engine.version(),
            &provider_voice_id,
            &segment.text,
            &segment.emotion,
            segment.rate_percent,
            segment.pitch_hz,
            segment.volume_percent,
        );

        if !payload.force {
            if let Some(hit) = cache::get(&state.db, &key)
                .await
                .map_err(|error| error.to_string())?
            {
                let mut tx = state.db.begin().await.map_err(|error| error.to_string())?;
                segments_repo::mark_done_tx(
                    &mut tx,
                    segment.id,
                    hit.file_id,
                    hit.duration_ms,
                    &key,
                )
                .await
                .map_err(|error| error.to_string())?;
                tx.commit().await.map_err(|error| error.to_string())?;
                let _ = cache::touch(&state.db, &key).await;
                cached += 1;
                done += 1;
                report_progress(state, job, done + failed, total, failed, "命中缓存").await;
                continue;
            }
        }

        let mut last_error = String::new();
        let mut succeeded = false;
        for attempt in 1..=job.max_attempts.max(1) {
            if jobs_repo::is_cancel_requested(&state.db, job.id)
                .await
                .map_err(|error| error.to_string())?
            {
                return Err("任务已取消".to_string());
            }

            let request = SynthesisRequest {
                text: segment.text.clone(),
                provider_voice_id: provider_voice_id.clone(),
                rate_percent: segment.rate_percent,
                pitch_hz: segment.pitch_hz,
                volume_percent: segment.volume_percent,
            };
            match engine.synthesize(&request).await {
                Ok(audio) => {
                    let duration_ms = ffmpeg::probe_duration_ms(
                        &state.config.audio.ffprobe_path,
                        &state.config.audio.tmp_dir,
                        &audio.bytes,
                    )
                    .await
                    .unwrap_or(0);
                    let object_key = format!(
                        "audio/segments/{}/{}/{}-{}.mp3",
                        book.id,
                        chapter.id,
                        segment.id,
                        &key[..8]
                    );
                    state
                        .storage
                        .put(&object_key, audio.bytes.clone(), "audio/mpeg")
                        .await
                        .map_err(|error| error.to_string())?;
                    let sha256 = hex::encode(Sha256::digest(&audio.bytes));
                    let file_id = files_service::register(
                        state,
                        NewFile {
                            bucket: state.storage.bucket().to_string(),
                            object_key,
                            file_name: format!("segment-{}.mp3", segment.id),
                            mime: "audio/mpeg".to_string(),
                            size_bytes: audio.bytes.len() as u64,
                            sha256,
                            kind: "segment_audio".to_string(),
                            duration_ms: Some(duration_ms),
                        },
                    )
                    .await
                    .map_err(|error| error.to_string())?;

                    let mut tx = state.db.begin().await.map_err(|error| error.to_string())?;
                    cache::put_tx(
                        &mut tx,
                        &NewCacheEntry {
                            cache_key: key.clone(),
                            provider: voice.provider.clone(),
                            provider_voice_id: provider_voice_id.clone(),
                            text_sha256: cache::text_sha256(&segment.text),
                            params_json: CacheParams {
                                emotion: &segment.emotion,
                                rate_percent: segment.rate_percent,
                                pitch_hz: segment.pitch_hz,
                                volume_percent: segment.volume_percent,
                                format: "mp3",
                            }
                            .to_json(),
                            file_id,
                            duration_ms,
                        },
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                    segments_repo::mark_done_tx(&mut tx, segment.id, file_id, duration_ms, &key)
                        .await
                        .map_err(|error| error.to_string())?;
                    tx.commit().await.map_err(|error| error.to_string())?;

                    done += 1;
                    succeeded = true;
                    break;
                }
                Err(error) => {
                    last_error = error.to_string();
                    if attempt < job.max_attempts {
                        tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                    }
                }
            }
        }

        if !succeeded {
            segments_repo::mark_failed(&state.db, segment.id, job.max_attempts, &last_error)
                .await
                .map_err(|error| error.to_string())?;
            failed += 1;
        }

        report_progress(
            state,
            job,
            done + failed,
            total,
            failed,
            &format!("合成 {}/{}", done + failed, total),
        )
        .await;
    }

    let (total_segments, done_segments, failed_segments) =
        segments_repo::chapter_counts(&state.db, chapter.id)
            .await
            .map_err(|error| error.to_string())?;
    let status = if done_segments == total_segments && failed_segments == 0 {
        "tts_done"
    } else {
        "tts_partial"
    };
    chapters_service::update_status(state, chapter.id, status)
        .await
        .map_err(|error| error.to_string())?;
    chapters_service::invalidate_after_tts(state, chapter.id)
        .await
        .map_err(|error| error.to_string())?;

    Ok(TtsRunResult {
        done,
        failed,
        cached,
    })
}

async fn report_progress(
    state: &AppState,
    job: &Job,
    processed: u64,
    total: u64,
    failed: u64,
    step: &str,
) {
    let progress = ((processed * 90 / total) as u8).min(90);
    let _ = jobs_repo::update_progress(
        &state.db,
        job.id,
        progress,
        processed.saturating_sub(failed) as u32,
        failed as u32,
        step,
    )
    .await;
}
