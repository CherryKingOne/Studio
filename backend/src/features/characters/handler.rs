use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;

use crate::common::api;
use crate::error::AppError;
use crate::features::characters::dto::{
    CreateCharacterRequest, DetectRequest, UpdateCharacterRequest,
};
use crate::features::characters::service;
use crate::middleware::auth::AuthUser;
use crate::state::AppState;

pub async fn list(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(book_id): Path<u64>,
) -> Result<Json<Value>, AppError> {
    let response = service::list(&state, book_id).await?;
    Ok(api::ok(response))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(book_id): Path<u64>,
    Json(request): Json<CreateCharacterRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let character = service::create(&state, book_id, request).await?;
    Ok(api::created(character))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
    Json(request): Json<UpdateCharacterRequest>,
) -> Result<Json<Value>, AppError> {
    let character = service::update(&state, id, request).await?;
    Ok(api::ok(character))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<StatusCode, AppError> {
    service::delete(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn detect(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(book_id): Path<u64>,
    Json(request): Json<DetectRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let job_id =
        service::create_detect_job(&state, book_id, request.overwrite_existing, user.id).await?;
    Ok(api::accepted(serde_json::json!({ "job_id": job_id })))
}
