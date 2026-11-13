use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;

use crate::recycle_bin::dto::{
    BatchPurgeRequest, BatchRestoreRequest, EmptyRecycleBinQuery, RecycleBinListQuery,
    RestoreBookRequest,
};
use crate::recycle_bin::service;
use crate::runtime::error::AppError;
use crate::runtime::middleware::auth::AuthUser;
use crate::runtime::state::AppState;
use crate::shared::api;

pub async fn list(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Query(query): Query<RecycleBinListQuery>,
) -> Result<Json<Value>, AppError> {
    let page = service::list(&state, query).await?;
    Ok(api::ok(page))
}

pub async fn summary(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
) -> Result<Json<Value>, AppError> {
    let summary = service::summary(&state).await?;
    Ok(api::ok(summary))
}

pub async fn restore(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path((resource_type, id)): Path<(String, u64)>,
    request: Option<Json<RestoreBookRequest>>,
) -> Result<Json<Value>, AppError> {
    let request = request.map(|Json(request)| request).unwrap_or_default();
    let restored = service::restore(&state, &resource_type, id, request).await?;
    Ok(api::ok(restored))
}

pub async fn purge(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path((resource_type, id)): Path<(String, u64)>,
) -> Result<StatusCode, AppError> {
    service::purge(&state, &resource_type, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn batch_restore(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Json(request): Json<BatchRestoreRequest>,
) -> Result<Json<Value>, AppError> {
    let response = service::batch_restore(&state, request).await?;
    Ok(api::ok(response))
}

pub async fn batch_purge(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Json(request): Json<BatchPurgeRequest>,
) -> Result<Json<Value>, AppError> {
    let response = service::batch_purge(&state, request).await?;
    Ok(api::ok(response))
}

pub async fn empty(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Query(query): Query<EmptyRecycleBinQuery>,
) -> Result<Json<Value>, AppError> {
    let response = service::empty(&state, query).await?;
    Ok(api::ok(response))
}
