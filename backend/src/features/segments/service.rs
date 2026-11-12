use std::collections::HashSet;

use sqlx::{MySql, Transaction};

use crate::common::text::TextSegment;
use crate::error::{AppError, FieldError};
use crate::features::chapters::service as chapters_service;
use crate::features::segments::dto::{
    BatchUpdateRequest, BatchUpdateResponse, SegmentDto, SegmentListQuery, SegmentListResponse,
    UpdateSegmentRequest,
};
use crate::features::segments::model::Segment;
use crate::features::segments::repo::{self, SegmentPatch};
use crate::state::AppState;

pub async fn list_by_chapter(
    state: &AppState,
    chapter_id: u64,
    query: SegmentListQuery,
) -> Result<SegmentListResponse, AppError> {
    let _ = chapters_service::get_model(state, chapter_id).await?;
    let segments = repo::list_by_chapter(&state.db, chapter_id, &query).await?;
    let total = repo::count_by_chapter(&state.db, chapter_id).await?;
    Ok(SegmentListResponse {
        items: segments.into_iter().map(SegmentDto::from).collect(),
        total,
    })
}

pub async fn update(
    state: &AppState,
    id: u64,
    request: UpdateSegmentRequest,
) -> Result<SegmentDto, AppError> {
    let segment = get_model(state, id).await?;
    let patch = build_patch(&segment, &request)?;

    let mut tx = state.db.begin().await?;
    repo::update_fields_tx(&mut tx, id, &patch).await?;
    chapters_service::clear_audio_tx(&mut tx, segment.chapter_id).await?;
    tx.commit().await?;

    get_model(state, id).await.map(SegmentDto::from)
}

pub async fn batch_update(
    state: &AppState,
    request: BatchUpdateRequest,
) -> Result<BatchUpdateResponse, AppError> {
    if request.items.is_empty() || request.items.len() > 200 {
        return Err(validation("items", "批量更新数量必须在 1 到 200 之间"));
    }

    let mut seen = HashSet::new();
    for item in &request.items {
        if !seen.insert(item.id) {
            return Err(validation("items", "存在重复的段落 ID"));
        }
    }

    let mut patches = Vec::with_capacity(request.items.len());
    let mut chapter_ids = HashSet::new();
    for item in request.items {
        let segment = get_model(state, item.id).await?;
        chapter_ids.insert(segment.chapter_id);
        patches.push((segment.id, build_patch(&segment, &item.patch)?));
    }

    let mut tx = state.db.begin().await?;
    for (id, patch) in &patches {
        repo::update_fields_tx(&mut tx, *id, patch).await?;
    }
    for chapter_id in chapter_ids {
        chapters_service::clear_audio_tx(&mut tx, chapter_id).await?;
    }
    tx.commit().await?;

    Ok(BatchUpdateResponse {
        updated: patches.len() as u64,
    })
}

pub async fn bulk_create_tx(
    tx: &mut Transaction<'_, MySql>,
    chapter_id: u64,
    segments: &[TextSegment],
) -> Result<u64, AppError> {
    Ok(repo::insert_batch_tx(tx, chapter_id, segments).await?)
}

pub async fn delete_by_chapter_tx(
    tx: &mut Transaction<'_, MySql>,
    chapter_id: u64,
) -> Result<(), AppError> {
    repo::delete_by_chapter_tx(tx, chapter_id).await?;
    Ok(())
}

pub async fn has_done_segments(state: &AppState, chapter_id: u64) -> Result<bool, AppError> {
    Ok(repo::has_done_by_chapter(&state.db, chapter_id).await?)
}

async fn get_model(state: &AppState, id: u64) -> Result<Segment, AppError> {
    repo::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound { resource: "段落" })
}

fn build_patch(
    segment: &Segment,
    request: &UpdateSegmentRequest,
) -> Result<SegmentPatch, AppError> {
    if segment.status == "synthesizing" {
        return Err(AppError::InvalidState(
            "段落正在合成中，暂不能编辑".to_string(),
        ));
    }

    let text = match request.text.as_deref() {
        Some(value) => {
            let value = value.trim();
            if value.is_empty() {
                return Err(validation("text", "段落文本不能为空"));
            }
            if value.chars().count() > 2000 {
                return Err(validation("text", "段落文本不能超过 2000 个字符"));
            }
            value.to_string()
        }
        None => segment.text.clone(),
    };

    let seg_type = match request.seg_type.as_deref() {
        Some(value) if !matches!(value, "narration" | "dialogue") => {
            return Err(validation("seg_type", "只能是 narration 或 dialogue"));
        }
        Some(value) => value.to_string(),
        None => segment.seg_type.clone(),
    };

    let speaker_id = request.speaker_id.or(segment.speaker_id);
    let speaker_confidence = if request.speaker_id.is_some() {
        Some(1.0)
    } else {
        segment.speaker_confidence
    };

    let emotion = match request.emotion.as_deref() {
        Some(value) => {
            let value = value.trim();
            if value.is_empty() || value.chars().count() > 24 {
                return Err(validation("emotion", "情绪长度必须在 1 到 24 个字符之间"));
            }
            value.to_string()
        }
        None => segment.emotion.clone(),
    };

    let rate_percent = request.rate_percent.unwrap_or(segment.rate_percent);
    let pitch_hz = request.pitch_hz.unwrap_or(segment.pitch_hz);
    let volume_percent = request.volume_percent.unwrap_or(segment.volume_percent);

    if !(-50..=100).contains(&rate_percent) {
        return Err(validation("rate_percent", "必须在 -50 到 100 之间"));
    }
    if !(-50..=50).contains(&pitch_hz) {
        return Err(validation("pitch_hz", "必须在 -50 到 50 之间"));
    }
    if !(-50..=50).contains(&volume_percent) {
        return Err(validation("volume_percent", "必须在 -50 到 50 之间"));
    }

    Ok(SegmentPatch {
        char_count: text.chars().count() as u32,
        text,
        seg_type,
        speaker_id,
        speaker_confidence,
        voice_id: request.voice_id.or(segment.voice_id),
        emotion,
        rate_percent,
        pitch_hz,
        volume_percent,
    })
}

fn validation(field: &str, message: &str) -> AppError {
    AppError::Validation(vec![FieldError {
        field: field.to_string(),
        message: message.to_string(),
    }])
}
