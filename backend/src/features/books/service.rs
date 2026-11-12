use bytes::Bytes;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::common::pagination::Page;
use crate::error::{AppError, FieldError};
use crate::features::books::dto::{
    BookDetail, BookListQuery, BookSettings, BookStats, BookSummary, CoverResponse,
    CreateBookRequest, UpdateBookRequest,
};
use crate::features::books::model::Book;
use crate::features::books::repo;
use crate::features::files::repo::NewFile;
use crate::features::files::service as files_service;
use crate::state::AppState;

pub async fn list(state: &AppState, query: BookListQuery) -> Result<Page<BookSummary>, AppError> {
    let (books, total) = repo::list(&state.db, &query).await?;
    let items = books.iter().map(BookSummary::from).collect();
    Ok(Page::new(items, query.page(), query.page_size(), total))
}

pub async fn create(
    state: &AppState,
    input: CreateBookRequest,
    user_id: u64,
) -> Result<BookDetail, AppError> {
    let title = validate_title(&input.title)?;
    if repo::exists_title(&state.db, &title).await? {
        return Err(AppError::Duplicate("同名书籍已存在".to_string()));
    }

    let settings = default_settings_value()?;
    let author = normalize_optional(input.author);
    let description = normalize_optional(input.description);
    let id = repo::insert(
        &state.db,
        &title,
        author.as_deref(),
        description.as_deref(),
        &settings,
        user_id,
    )
    .await?;

    get_model(state, id).await.map(BookDetail::from)
}

pub async fn get(state: &AppState, id: u64) -> Result<BookDetail, AppError> {
    get_model(state, id).await.map(BookDetail::from)
}

pub async fn update(
    state: &AppState,
    id: u64,
    input: UpdateBookRequest,
) -> Result<BookDetail, AppError> {
    let existing = get_model(state, id).await?;

    let title = match input.title {
        Some(title) => {
            let title = validate_title(&title)?;
            if title != existing.title && repo::exists_title(&state.db, &title).await? {
                return Err(AppError::Duplicate("同名书籍已存在".to_string()));
            }
            title
        }
        None => existing.title.clone(),
    };

    let author = match input.author {
        Some(value) => normalize_optional(Some(value)),
        None => existing.author.clone(),
    };
    let description = match input.description {
        Some(value) => normalize_optional(Some(value)),
        None => existing.description.clone(),
    };
    let narrator_voice_id = input
        .default_narrator_voice_id
        .or(existing.default_narrator_voice_id);
    let settings = match input.settings {
        Some(settings) => serde_json::to_value(settings)?,
        None => existing.settings.0.clone(),
    };

    repo::update(
        &state.db,
        id,
        &title,
        author.as_deref(),
        description.as_deref(),
        narrator_voice_id,
        &settings,
    )
    .await?;

    get_model(state, id).await.map(BookDetail::from)
}

pub async fn delete(state: &AppState, id: u64) -> Result<(), AppError> {
    let _ = get_model(state, id).await?;
    if repo::count_active_jobs(&state.db, id).await? > 0 {
        return Err(AppError::InvalidState(
            "书籍存在进行中的任务，暂不能删除".to_string(),
        ));
    }
    repo::soft_delete(&state.db, id).await?;
    Ok(())
}

pub async fn stats(state: &AppState, id: u64) -> Result<BookStats, AppError> {
    let book = get_model(state, id).await?;
    let stats = repo::stats(&state.db, id).await?;
    Ok(BookStats {
        total_chapters: stats.total_chapters,
        segmented_chapters: stats.segmented_chapters,
        tts_done_chapters: stats.tts_done_chapters,
        merged_chapters: stats.merged_chapters,
        total_segments: stats.total_segments,
        done_segments: stats.done_segments,
        failed_segments: stats.failed_segments,
        total_duration_ms: book.total_duration_ms,
        active_job_count: stats.active_job_count,
    })
}

pub async fn upload_cover(
    state: &AppState,
    book_id: u64,
    file_name: &str,
    content_type: Option<&str>,
    bytes: Bytes,
) -> Result<CoverResponse, AppError> {
    let _ = get_model(state, book_id).await?;

    if bytes.len() > state.config.limits.max_image_bytes {
        return Err(AppError::FileTooLarge);
    }

    let (mime, extension) =
        image_info(content_type, file_name).ok_or(AppError::UnsupportedFileType)?;

    let size_bytes = bytes.len() as u64;
    let object_key = format!("covers/{book_id}/{}.{}", uuid::Uuid::now_v7(), extension);
    let sha256 = hex::encode(Sha256::digest(&bytes));

    state.storage.put(&object_key, bytes, mime).await?;

    let file_id = files_service::register(
        state,
        NewFile {
            bucket: state.storage.bucket().to_string(),
            object_key,
            file_name: file_name.to_string(),
            mime: mime.to_string(),
            size_bytes,
            sha256,
            kind: "cover".to_string(),
            duration_ms: None,
        },
    )
    .await?;

    repo::update_cover(&state.db, book_id, file_id).await?;

    Ok(CoverResponse {
        cover_file_id: file_id,
        cover_url: format!("/api/v1/files/{file_id}"),
    })
}

pub async fn get_model(state: &AppState, id: u64) -> Result<Book, AppError> {
    repo::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound { resource: "书籍" })
}

fn validate_title(title: &str) -> Result<String, AppError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::Validation(vec![FieldError {
            field: "title".to_string(),
            message: "不能为空".to_string(),
        }]));
    }
    if title.chars().count() > 255 {
        return Err(AppError::Validation(vec![FieldError {
            field: "title".to_string(),
            message: "不能超过 255 个字符".to_string(),
        }]));
    }
    Ok(title.to_string())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn default_settings_value() -> Result<Value, AppError> {
    Ok(serde_json::to_value(BookSettings::default())?)
}

fn image_info(content_type: Option<&str>, file_name: &str) -> Option<(&'static str, &'static str)> {
    if let Some(content_type) = content_type {
        match content_type {
            "image/jpeg" => return Some(("image/jpeg", "jpg")),
            "image/png" => return Some(("image/png", "png")),
            "image/webp" => return Some(("image/webp", "webp")),
            _ => {}
        }
    }

    let extension = file_name.rsplit('.').next()?.to_ascii_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" => Some(("image/jpeg", "jpg")),
        "png" => Some(("image/png", "png")),
        "webp" => Some(("image/webp", "webp")),
        _ => None,
    }
}
