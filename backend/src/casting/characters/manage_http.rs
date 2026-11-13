use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;

use crate::casting::characters::manage;
use crate::casting::characters::responses::{
    CreateCharacterRequest, DetectRequest, UpdateCharacterRequest,
};
use crate::runtime::error::AppError;
use crate::runtime::middleware::auth::AuthUser;
use crate::runtime::state::AppState;
use crate::shared::api;

pub async fn list(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(book_id): Path<u64>,
) -> Result<Json<Value>, AppError> {
    let response = manage::list(&state, book_id).await?;
    Ok(api::ok(response))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(book_id): Path<u64>,
    Json(request): Json<CreateCharacterRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let character = manage::create(&state, book_id, request).await?;
    Ok(api::created(character))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
    Json(request): Json<UpdateCharacterRequest>,
) -> Result<Json<Value>, AppError> {
    let character = manage::update(&state, id, request).await?;
    Ok(api::ok(character))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<StatusCode, AppError> {
    manage::delete(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn detect(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(book_id): Path<u64>,
    Json(request): Json<DetectRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let job_id =
        manage::create_detect_job(&state, book_id, request.overwrite_existing, user.id).await?;
    Ok(api::accepted(serde_json::json!({ "job_id": job_id })))
}
