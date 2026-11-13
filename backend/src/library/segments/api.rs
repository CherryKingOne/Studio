use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde_json::Value;

use crate::library::segments::segment_actions;
use crate::library::segments::segment_types::{
    BatchUpdateRequest, SegmentListQuery, UpdateSegmentRequest,
};
use crate::runtime::error::AppError;
use crate::runtime::middleware::auth::AuthUser;
use crate::runtime::state::AppState;
use crate::shared::api;

pub async fn list(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(chapter_id): Path<u64>,
    Query(query): Query<SegmentListQuery>,
) -> Result<Json<Value>, AppError> {
    let response = segment_actions::list_by_chapter(&state, chapter_id, query).await?;
    Ok(api::ok(response))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
    Json(request): Json<UpdateSegmentRequest>,
) -> Result<Json<Value>, AppError> {
    let segment = segment_actions::update(&state, id, request).await?;
    Ok(api::ok(segment))
}

pub async fn batch_update(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Json(request): Json<BatchUpdateRequest>,
) -> Result<Json<Value>, AppError> {
    let response = segment_actions::batch_update(&state, request).await?;
    Ok(api::ok(response))
}

pub fn routes() -> axum::Router<std::sync::Arc<crate::runtime::state::AppState>> {
    axum::Router::new()
        .route(
            "/api/v1/chapters/{chapter_id}/segments",
            axum::routing::get(list),
        )
        .route("/api/v1/segments/batch", axum::routing::patch(batch_update))
        .route("/api/v1/segments/{id}", axum::routing::patch(update))
}
