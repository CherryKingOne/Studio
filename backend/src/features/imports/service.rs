use std::path::Path;

use bytes::Bytes;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::error::{AppError, FieldError};
use crate::features::books::dto::CreateBookRequest;
use crate::features::books::repo as books_repo;
use crate::features::books::service as books_service;
use crate::features::chapters::repo as chapters_repo;
use crate::features::chapters::service as chapters_service;
use crate::features::files::repo::NewFile;
use crate::features::files::service as files_service;
use crate::features::imports::dto::{ImportPayload, ImportResponse};
use crate::features::imports::parser;
use crate::features::jobs::model::Job;
use crate::features::jobs::queue as job_queue;
use crate::features::jobs::repo as jobs_repo;
use crate::features::jobs::repo::NewJob;
use crate::features::segments::service as segments_service;
use crate::state::AppState;

pub async fn create_import(
    state: &AppState,
    user_id: u64,
    file_name: &str,
    title: Option<String>,
    author: Option<String>,
    bytes: Bytes,
) -> Result<ImportResponse, AppError> {
    if !file_name.to_ascii_lowercase().ends_with(".txt") {
        return Err(AppError::UnsupportedFileType);
    }
    if bytes.len() > state.config.limits.max_import_bytes {
        return Err(AppError::FileTooLarge);
    }

    let title = title
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            Path::new(file_name)
                .file_stem()
                .and_then(|value| value.to_str())
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "未命名书籍".to_string());

    if title.chars().count() > 255 {
        return Err(AppError::Validation(vec![FieldError {
            field: "title".to_string(),
            message: "书名不能超过 255 个字符".to_string(),
        }]));
    }

    let book = books_service::create(
        state,
        CreateBookRequest {
            title,
            author: author.filter(|value| !value.trim().is_empty()),
            description: None,
        },
        user_id,
    )
    .await?;

    let sha256 = hex::encode(Sha256::digest(&bytes));
    let object_key = format!("sources/{}/{}.txt", book.id, uuid::Uuid::now_v7());
    state
        .storage
        .put(&object_key, bytes.clone(), "text/plain")
        .await?;

    let file_id = match files_service::register(
        state,
        NewFile {
            bucket: state.storage.bucket().to_string(),
            object_key: object_key.clone(),
            file_name: file_name.to_string(),
            mime: "text/plain".to_string(),
            size_bytes: bytes.len() as u64,
            sha256,
            kind: "source".to_string(),
            duration_ms: None,
        },
    )
    .await
    {
        Ok(file_id) => file_id,
        Err(error) => {
            if let Err(cleanup_error) = state.storage.delete(&object_key).await {
                tracing::warn!(%cleanup_error, "cleanup source object failed");
            }
            return Err(error);
        }
    };

    let payload = ImportPayload {
        book_id: book.id,
        file_id,
        max_chars: state.config.limits.import_max_chars,
    };
    let new_job = NewJob {
        job_type: "import_parse".to_string(),
        priority: 7,
        book_id: Some(book.id),
        chapter_id: None,
        created_by: Some(user_id),
        payload: serde_json::to_value(&payload)?,
        total_items: 0,
        max_attempts: 3,
    };

    let mut tx = state.db.begin().await?;
    books_repo::update_import_source_tx(&mut tx, book.id, file_id).await?;
    let job_id = job_queue::enqueue_tx(&mut tx, &new_job, &[]).await?;
    tx.commit().await?;

    Ok(ImportResponse {
        book_id: book.id,
        job_id,
    })
}

pub async fn run_import(state: &AppState, job: &Job) -> Result<Value, String> {
    let payload: ImportPayload = serde_json::from_value(job.payload.0.clone())
        .map_err(|error| format!("任务参数解析失败: {error}"))?;

    books_repo::update_status(&state.db, payload.book_id, "parsing", None)
        .await
        .map_err(|error| error.to_string())?;
    jobs_repo::update_progress(&state.db, job.id, 5, 0, 0, "读取源文件")
        .await
        .map_err(|error| error.to_string())?;

    let file = files_service::get(state, payload.file_id)
        .await
        .map_err(|error| error.to_string())?;
    let bytes = state
        .storage
        .get(&file.object_key)
        .await
        .map_err(|error| error.to_string())?;

    let parsed = match parser::parse_book(&bytes, payload.max_chars) {
        Ok(parsed) => parsed,
        Err(error) => {
            let _ =
                books_repo::update_status(&state.db, payload.book_id, "failed", Some(&error)).await;
            return Err(error);
        }
    };

    jobs_repo::set_total_items(&state.db, job.id, parsed.chapters.len() as u32)
        .await
        .map_err(|error| error.to_string())?;
    chapters_repo::delete_by_book(&state.db, payload.book_id)
        .await
        .map_err(|error| error.to_string())?;
    books_repo::update_status(&state.db, payload.book_id, "segmenting", None)
        .await
        .map_err(|error| error.to_string())?;

    let total_chapters = parsed.chapters.len();
    for (position, chapter) in parsed.chapters.iter().enumerate() {
        if jobs_repo::is_cancel_requested(&state.db, job.id)
            .await
            .map_err(|error| error.to_string())?
        {
            let _ =
                books_repo::update_status(&state.db, payload.book_id, "failed", Some("任务已取消"))
                    .await;
            return Err("任务已取消".to_string());
        }

        let mut tx = state.db.begin().await.map_err(|error| error.to_string())?;
        let chapter_id = chapters_service::create_tx(
            &mut tx,
            payload.book_id,
            chapter.index,
            &chapter.title,
            &chapter.raw_text,
        )
        .await
        .map_err(|error| error.to_string())?;
        segments_service::bulk_create_tx(&mut tx, chapter_id, &chapter.segments)
            .await
            .map_err(|error| error.to_string())?;
        tx.commit().await.map_err(|error| error.to_string())?;

        let progress = 10 + ((position + 1) * 80 / total_chapters) as u8;
        jobs_repo::update_progress(
            &state.db,
            job.id,
            progress.min(90),
            (position + 1) as u32,
            0,
            &format!("切分章节 {}/{}", position + 1, total_chapters),
        )
        .await
        .map_err(|error| error.to_string())?;
    }

    books_repo::update_stats(
        &state.db,
        payload.book_id,
        total_chapters as u32,
        parsed.total_segments,
        "ready",
    )
    .await
    .map_err(|error| error.to_string())?;
    jobs_repo::update_progress(&state.db, job.id, 100, total_chapters as u32, 0, "导入完成")
        .await
        .map_err(|error| error.to_string())?;

    Ok(json!({
        "book_id": payload.book_id,
        "chapters": total_chapters,
        "segments": parsed.total_segments,
        "encoding": format!("{:?}", parsed.encoding),
    }))
}
