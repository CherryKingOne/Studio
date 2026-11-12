use sqlx::{MySql, Transaction};

use crate::common::pagination::Page;
use crate::common::text;
use crate::error::{AppError, FieldError};
use crate::features::chapters::dto::{
    ChapterDetail, ChapterListQuery, ChapterSummary, ResegmentResponse, UpdateChapterRequest,
};
use crate::features::chapters::model::Chapter;
use crate::features::chapters::repo;
use crate::features::segments::service as segments_service;
use crate::state::AppState;

pub async fn list(
    state: &AppState,
    book_id: u64,
    query: ChapterListQuery,
) -> Result<Page<ChapterSummary>, AppError> {
    let (rows, total) = repo::list(&state.db, book_id, &query).await?;
    let items = rows.iter().map(ChapterSummary::from).collect();
    Ok(Page::new(items, query.page(), query.page_size(), total))
}

pub async fn get(state: &AppState, id: u64) -> Result<ChapterDetail, AppError> {
    get_model(state, id).await.map(ChapterDetail::from)
}

pub async fn get_model(state: &AppState, id: u64) -> Result<Chapter, AppError> {
    repo::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound { resource: "章节" })
}

pub async fn update(
    state: &AppState,
    id: u64,
    request: UpdateChapterRequest,
) -> Result<ChapterDetail, AppError> {
    let chapter = get_model(state, id).await?;

    let title = match request.title {
        Some(title) => validate_title(&title)?,
        None => chapter.title.clone(),
    };

    let (raw_text, char_count, status) = match request.raw_text {
        Some(raw_text) => {
            if chapter.audio_file_id.is_some()
                || segments_service::has_done_segments(state, id).await?
            {
                return Err(AppError::InvalidState(
                    "章节已有合成音频，不能直接修改正文".to_string(),
                ));
            }
            let raw_text = raw_text.trim().to_string();
            if raw_text.is_empty() {
                return Err(AppError::Validation(vec![FieldError {
                    field: "raw_text".to_string(),
                    message: "正文不能为空".to_string(),
                }]));
            }
            let char_count = raw_text.chars().count() as u32;
            (raw_text, char_count, "segmented".to_string())
        }
        None => (
            chapter.raw_text.clone(),
            chapter.char_count,
            chapter.status.clone(),
        ),
    };

    repo::update_content(&state.db, id, &title, &raw_text, char_count, &status).await?;

    if title != chapter.title || chapter.audio_file_id.is_some() {
        let mut tx = state.db.begin().await?;
        repo::clear_audio_tx(&mut tx, id).await?;
        tx.commit().await?;
    }

    get(state, id).await
}

pub async fn resegment(
    state: &AppState,
    id: u64,
    confirm: bool,
    max_chars: Option<usize>,
) -> Result<ResegmentResponse, AppError> {
    if !confirm {
        return Err(AppError::Validation(vec![FieldError {
            field: "confirm".to_string(),
            message: "必须确认重新切段".to_string(),
        }]));
    }

    let chapter = get_model(state, id).await?;
    if chapter.audio_file_id.is_some() || segments_service::has_done_segments(state, id).await? {
        return Err(AppError::InvalidState(
            "章节已有合成音频，不能重新切段".to_string(),
        ));
    }

    let max_chars = max_chars.unwrap_or(state.config.limits.import_max_chars);
    if !(50..=1000).contains(&max_chars) {
        return Err(AppError::Validation(vec![FieldError {
            field: "max_chars".to_string(),
            message: "max_chars 必须在 50 到 1000 之间".to_string(),
        }]));
    }

    let segments = text::split_segments(&chapter.raw_text, max_chars);
    if segments.is_empty() {
        return Err(AppError::InvalidState("正文无法切分出段落".to_string()));
    }

    let char_count = chapter.raw_text.chars().count() as u32;
    let mut tx = state.db.begin().await?;
    segments_service::delete_by_chapter_tx(&mut tx, id).await?;
    segments_service::bulk_create_tx(&mut tx, id, &segments).await?;
    repo::update_after_resegment_tx(&mut tx, id, char_count).await?;
    tx.commit().await?;

    Ok(ResegmentResponse {
        segments_created: segments.len() as u64,
        char_count,
    })
}

pub async fn create_tx(
    tx: &mut Transaction<'_, MySql>,
    book_id: u64,
    idx: u32,
    title: &str,
    raw_text: &str,
) -> Result<u64, AppError> {
    let char_count = raw_text.chars().count() as u32;
    Ok(repo::insert_tx(tx, book_id, idx, title, raw_text, char_count).await?)
}

pub async fn clear_audio_tx(
    tx: &mut Transaction<'_, MySql>,
    chapter_id: u64,
) -> Result<(), AppError> {
    repo::clear_audio_tx(tx, chapter_id).await?;
    Ok(())
}

pub async fn clear_audio(state: &AppState, chapter_id: u64) -> Result<(), AppError> {
    let mut tx = state.db.begin().await?;
    repo::clear_audio_tx(&mut tx, chapter_id).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn update_status(
    state: &AppState,
    chapter_id: u64,
    status: &str,
) -> Result<(), AppError> {
    repo::update_status(&state.db, chapter_id, status).await?;
    Ok(())
}

pub async fn invalidate_after_tts(state: &AppState, chapter_id: u64) -> Result<(), AppError> {
    repo::invalidate_after_tts(&state.db, chapter_id).await?;
    Ok(())
}

fn validate_title(title: &str) -> Result<String, AppError> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 255 {
        return Err(AppError::Validation(vec![FieldError {
            field: "title".to_string(),
            message: "标题长度必须在 1 到 255 个字符之间".to_string(),
        }]));
    }
    Ok(title.to_string())
}
