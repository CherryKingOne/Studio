use std::sync::Arc;

use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;

use crate::common::api;
use crate::error::{AppError, FieldError};
use crate::features::books::dto::{BookListQuery, CreateBookRequest, UpdateBookRequest};
use crate::features::books::service;
use crate::middleware::auth::AuthUser;
use crate::state::AppState;

pub async fn list(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(query): Query<BookListQuery>,
) -> Result<Json<Value>, AppError> {
    let _ = user;
    let page = service::list(&state, query).await?;
    Ok(api::ok(page))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(request): Json<CreateBookRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let book = service::create(&state, request, user.id).await?;
    Ok(api::created(book))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<Json<Value>, AppError> {
    let book = service::get(&state, id).await?;
    Ok(api::ok(book))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
    Json(request): Json<UpdateBookRequest>,
) -> Result<Json<Value>, AppError> {
    let book = service::update(&state, id, request).await?;
    Ok(api::ok(book))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<StatusCode, AppError> {
    service::delete(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn stats(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<Json<Value>, AppError> {
    let stats = service::stats(&state, id).await?;
    Ok(api::ok(stats))
}

pub async fn upload_cover(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|error| {
        AppError::Validation(vec![FieldError {
            field: "file".to_string(),
            message: error.to_string(),
        }])
    })? {
        if field.name() != Some("file") {
            continue;
        }

        let file_name = field.file_name().unwrap_or("cover").to_string();
        let content_type = field.content_type().map(str::to_string);
        let bytes = field.bytes().await.map_err(|error| {
            AppError::Validation(vec![FieldError {
                field: "file".to_string(),
                message: error.to_string(),
            }])
        })?;

        let response =
            service::upload_cover(&state, id, &file_name, content_type.as_deref(), bytes).await?;
        return Ok(api::ok(response));
    }

    Err(AppError::Validation(vec![FieldError {
        field: "file".to_string(),
        message: "缺少上传文件".to_string(),
    }]))
}
