use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde_json::Value;

use crate::common::api;
use crate::error::AppError;
use crate::features::chapters::dto::{ChapterListQuery, ResegmentRequest, UpdateChapterRequest};
use crate::features::chapters::service;
use crate::middleware::auth::AuthUser;
use crate::state::AppState;

pub async fn list(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(book_id): Path<u64>,
    Query(query): Query<ChapterListQuery>,
) -> Result<Json<Value>, AppError> {
    let page = service::list(&state, book_id, query).await?;
    Ok(api::ok(page))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<Json<Value>, AppError> {
    let chapter = service::get(&state, id).await?;
    Ok(api::ok(chapter))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
    Json(request): Json<UpdateChapterRequest>,
) -> Result<Json<Value>, AppError> {
    let chapter = service::update(&state, id, request).await?;
    Ok(api::ok(chapter))
}

pub async fn resegment(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
    Json(request): Json<ResegmentRequest>,
) -> Result<Json<Value>, AppError> {
    let response = service::resegment(&state, id, request.confirm, request.max_chars).await?;
    Ok(api::ok(response))
}
