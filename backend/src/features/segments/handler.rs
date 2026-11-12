use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde_json::Value;

use crate::common::api;
use crate::error::AppError;
use crate::features::segments::dto::{BatchUpdateRequest, SegmentListQuery, UpdateSegmentRequest};
use crate::features::segments::service;
use crate::middleware::auth::AuthUser;
use crate::state::AppState;

pub async fn list(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(chapter_id): Path<u64>,
    Query(query): Query<SegmentListQuery>,
) -> Result<Json<Value>, AppError> {
    let response = service::list_by_chapter(&state, chapter_id, query).await?;
    Ok(api::ok(response))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
    Json(request): Json<UpdateSegmentRequest>,
) -> Result<Json<Value>, AppError> {
    let segment = service::update(&state, id, request).await?;
    Ok(api::ok(segment))
}

pub async fn batch_update(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Json(request): Json<BatchUpdateRequest>,
) -> Result<Json<Value>, AppError> {
    let response = service::batch_update(&state, request).await?;
    Ok(api::ok(response))
}
