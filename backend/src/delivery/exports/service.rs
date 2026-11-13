use std::io::Write;
use std::path::PathBuf;

use bytes::Bytes;
use serde_json::json;
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

use crate::delivery::exports::dto::{
    CreateExportRequest, CreateExportResponse, ExportDto, ExportOptions, ExportPayload,
    ExportRunResult,
};
use crate::delivery::exports::repo;
use crate::delivery::files::repo::NewFile;
use crate::delivery::files::service as files_service;
use crate::library::books::service as books_service;
use crate::library::chapters::repo as chapters_repo;
use crate::runtime::error::AppError;
use crate::runtime::state::AppState;
use crate::shared::temp::TempDir;
use crate::shared::text::sanitize_filename;
use crate::task_center::model::Job;
use crate::task_center::queue as jobs_queue;
use crate::task_center::repo as jobs_repo;
use crate::task_center::repo::NewJob;

pub async fn create_export(
    state: &AppState,
    book_id: u64,
    request: CreateExportRequest,
    user_id: u64,
) -> Result<CreateExportResponse, AppError> {
    let book = books_service::get_model(state, book_id).await?;
    if let Some(existing_job_id) =
        jobs_repo::active_job_id(&state.db, book_id, "book_export").await?
    {
        return Err(AppError::JobAlreadyActive { existing_job_id });
    }
    let merged = chapters_repo::list_merged_by_book(&state.db, book_id).await?;
    if merged.is_empty() {
        return Err(AppError::InvalidState(
            "没有已拼接章节，无法导出".to_string(),
        ));
    }

    let chapter_ids = request.chapter_ids;
    let selected = match &chapter_ids {
        Some(ids) => {
            let selected: Vec<_> = merged
                .into_iter()
                .filter(|chapter| ids.contains(&chapter.id))
                .collect();
            if selected.len() != ids.len() {
                return Err(AppError::InvalidState("部分章节未拼接或不存在".to_string()));
            }
            selected
        }
        None => merged,
    };
    if selected.is_empty() {
        return Err(AppError::InvalidState("没有可导出章节".to_string()));
    }

    let include_cover = request.include_cover.unwrap_or(true);
    let options = ExportOptions {
        include_cover,
        chapter_ids: chapter_ids.clone(),
    };
    let export_id = repo::insert(
        &state.db,
        book.id,
        &serde_json::to_value(&options)?,
        selected.len() as u32,
    )
    .await?;

    let payload = ExportPayload {
        export_id,
        book_id: book.id,
        chapter_ids,
        include_cover,
    };
    let job = NewJob {
        job_type: "book_export".to_string(),
        priority: 3,
        book_id: Some(book.id),
        chapter_id: None,
        created_by: Some(user_id),
        payload: serde_json::to_value(&payload)?,
        total_items: selected.len() as u32,
        max_attempts: 2,
    };
    let job_id = jobs_queue::enqueue(&state.db, &job, &[]).await?;
    Ok(CreateExportResponse { export_id, job_id })
}

pub async fn run_export(state: &AppState, job: &Job) -> Result<ExportRunResult, String> {
    let payload: ExportPayload =
        serde_json::from_value(job.payload.0.clone()).map_err(|error| error.to_string())?;
    let _ = repo::mark_running(&state.db, payload.export_id).await;
    let book = books_service::get_model(state, payload.book_id)
        .await
        .map_err(|error| error.to_string())?;
    let merged = chapters_repo::list_merged_by_book(&state.db, payload.book_id)
        .await
        .map_err(|error| error.to_string())?;
    let chapters: Vec<_> = match &payload.chapter_ids {
        Some(ids) => merged
            .into_iter()
            .filter(|chapter| ids.contains(&chapter.id))
            .collect(),
        None => merged,
    };
    if chapters.is_empty() {
        return Err("没有可导出章节".to_string());
    }

    let tmp = TempDir::create(&state.config.audio.tmp_dir, &format!("export-{}", job.id))
        .await
        .map_err(|error| error.to_string())?;

    let mut used_names = std::collections::HashSet::new();
    let mut chapter_files: Vec<(String, PathBuf, serde_json::Value)> = Vec::new();
    let mut total_duration_ms = 0u64;

    for (index, chapter) in chapters.iter().enumerate() {
        let file_id = chapter
            .audio_file_id
            .ok_or_else(|| format!("章节 {} 缺少音频", chapter.id))?;
        let file = files_service::get(state, file_id)
            .await
            .map_err(|error| error.to_string())?;
        let bytes = state
            .storage
            .get(&file.object_key)
            .await
            .map_err(|error| error.to_string())?;
        let mut file_name = format!(
            "{:03}-{}.mp3",
            chapter.idx,
            sanitize_filename(&chapter.title)
        );
        if !used_names.insert(file_name.clone()) {
            file_name = format!(
                "{:03}-{}-{}.mp3",
                chapter.idx,
                sanitize_filename(&chapter.title),
                chapter.id
            );
        }
        let path = tmp.path().join(&file_name);
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(|error| error.to_string())?;
        if let Some(duration) = chapter.audio_duration_ms {
            total_duration_ms += duration as u64;
        }
        chapter_files.push((
            file_name.clone(),
            path,
            json!({
                "index": chapter.idx,
                "title": chapter.title,
                "file": file_name,
                "duration_ms": chapter.audio_duration_ms,
            }),
        ));

        let progress = 10 + ((index + 1) * 40 / chapters.len()) as u8;
        let _ = jobs_repo::update_progress(
            &state.db,
            job.id,
            progress,
            0,
            0,
            &format!("下载章节音频 {}/{}", index + 1, chapters.len()),
        )
        .await;
    }

    let mut cover_path = None;
    if payload.include_cover {
        if let Some(cover_file_id) = book.cover_file_id {
            if let Ok(file) = files_service::get(state, cover_file_id).await {
                if let Ok(bytes) = state.storage.get(&file.object_key).await {
                    let extension = file
                        .file_name
                        .rsplit('.')
                        .next()
                        .unwrap_or("jpg")
                        .to_ascii_lowercase();
                    let path = tmp.path().join(format!("cover.{extension}"));
                    if tokio::fs::write(&path, &bytes).await.is_ok() {
                        cover_path = Some(path);
                    }
                }
            }
        }
    }

    let manifest = json!({
        "book": {
            "id": book.id,
            "title": book.title,
            "author": book.author,
        },
        "exported_at": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "generator": format!("NovelCast {}", env!("CARGO_PKG_VERSION")),
        "total_duration_ms": total_duration_ms,
        "chapters": chapter_files.iter().map(|(_, _, entry)| entry.clone()).collect::<Vec<_>>(),
    });
    let manifest_path = tmp.path().join("manifest.json");
    tokio::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?,
    )
    .await
    .map_err(|error| error.to_string())?;

    let zip_path = tmp.path().join("export.zip");
    let zip_file = std::fs::File::create(&zip_path).map_err(|error| error.to_string())?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    zip.start_file("manifest.json", options)
        .map_err(|error| error.to_string())?;
    zip.write_all(
        &tokio::fs::read(&manifest_path)
            .await
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    for (name, path, _) in &chapter_files {
        zip.start_file(name, options)
            .map_err(|error| error.to_string())?;
        zip.write_all(
            &tokio::fs::read(path)
                .await
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    }

    if let Some(path) = &cover_path {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("cover.jpg");
        zip.start_file(name, options)
            .map_err(|error| error.to_string())?;
        zip.write_all(
            &tokio::fs::read(path)
                .await
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    }
    zip.finish().map_err(|error| error.to_string())?;

    let bytes = tokio::fs::read(&zip_path)
        .await
        .map_err(|error| error.to_string())?;
    let size_bytes = bytes.len() as u64;
    let object_key = format!("exports/{}/{}.zip", book.id, payload.export_id);
    state
        .storage
        .put(&object_key, Bytes::from(bytes.clone()), "application/zip")
        .await
        .map_err(|error| error.to_string())?;
    let sha256 = hex::encode(Sha256::digest(&bytes));
    let file_id = files_service::register(
        state,
        NewFile {
            bucket: state.storage.bucket().to_string(),
            object_key,
            file_name: format!("{}.zip", sanitize_filename(&book.title)),
            mime: "application/zip".to_string(),
            size_bytes,
            sha256,
            kind: "export_zip".to_string(),
            duration_ms: None,
        },
    )
    .await
    .map_err(|error| error.to_string())?;

    let included = chapter_files.len() as u32;
    repo::mark_succeeded(&state.db, payload.export_id, file_id, included)
        .await
        .map_err(|error| error.to_string())?;
    let _ = jobs_repo::update_progress(&state.db, job.id, 100, included, 0, "导出完成").await;

    Ok(ExportRunResult {
        export_id: payload.export_id,
        file_id,
        included_chapters: included,
        size_bytes,
    })
}

pub async fn get_export(state: &AppState, id: u64) -> Result<ExportDto, AppError> {
    let row = repo::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound {
            resource: "导出记录",
        })?;
    Ok(ExportDto::from(row))
}

pub async fn download(state: &AppState, id: u64) -> Result<(Bytes, String), AppError> {
    let row = repo::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound {
            resource: "导出记录",
        })?;
    if row.status != "succeeded" {
        return Err(AppError::InvalidState("导出尚未完成".to_string()));
    }
    let file_id = row.file_id.ok_or(AppError::NotFound {
        resource: "导出文件",
    })?;
    let file = files_service::get(state, file_id).await?;
    let bytes = state.storage.get(&file.object_key).await?;
    let book = books_service::get_model(state, row.book_id).await?;
    Ok((bytes, format!("{}.zip", sanitize_filename(&book.title))))
}

pub async fn mark_failed(state: &AppState, export_id: u64, error: &str) {
    let _ = repo::mark_failed(&state.db, export_id, error).await;
}
