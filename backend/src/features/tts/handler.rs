use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde_json::Value;

use crate::common::api;
use crate::error::AppError;
use crate::features::tts::dto::{ChapterTtsRequest, SegmentTtsRequest};
use crate::features::tts::service;
use crate::middleware::auth::AuthUser;
use crate::state::AppState;

pub async fn chapter_tts(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(chapter_id): Path<u64>,
    Json(request): Json<ChapterTtsRequest>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    let mode = request.mode.as_deref().unwrap_or("missing");
    let response =
        service::create_chapter_job(&state, chapter_id, mode, request.force, user.id).await?;
    Ok(api::accepted(response))
}

pub async fn segment_tts(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(segment_id): Path<u64>,
    Json(request): Json<SegmentTtsRequest>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    let response = service::create_segment_job(&state, segment_id, request.force, user.id).await?;
    Ok(api::accepted(response))
}
