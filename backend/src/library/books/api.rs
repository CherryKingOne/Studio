use std::sync::Arc;

use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;

use crate::library::books::book_actions;
use crate::library::books::book_types::{BookListQuery, CreateBookRequest, UpdateBookRequest};
use crate::runtime::error::{AppError, FieldError};
use crate::runtime::middleware::auth::AuthUser;
use crate::runtime::state::AppState;
use crate::shared::api;

pub async fn list(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(query): Query<BookListQuery>,
) -> Result<Json<Value>, AppError> {
    let _ = user;
    let page = book_actions::list(&state, query).await?;
    Ok(api::ok(page))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(request): Json<CreateBookRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let book = book_actions::create(&state, request, user.id).await?;
    Ok(api::created(book))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<Json<Value>, AppError> {
    let book = book_actions::get(&state, id).await?;
    Ok(api::ok(book))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
    Json(request): Json<UpdateBookRequest>,
) -> Result<Json<Value>, AppError> {
    let book = book_actions::update(&state, id, request).await?;
    Ok(api::ok(book))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> Result<StatusCode, AppError> {
    book_actions::delete(&state, id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn stats(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<Json<Value>, AppError> {
    let stats = book_actions::stats(&state, id).await?;
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
            book_actions::upload_cover(&state, id, &file_name, content_type.as_deref(), bytes)
                .await?;
        return Ok(api::ok(response));
    }

    Err(AppError::Validation(vec![FieldError {
        field: "file".to_string(),
        message: "缺少上传文件".to_string(),
    }]))
}

pub fn routes() -> axum::Router<std::sync::Arc<crate::runtime::state::AppState>> {
    axum::Router::new()
        .route("/api/v1/books", axum::routing::get(list).post(create))
        .route(
            "/api/v1/books/{id}",
            axum::routing::get(get).patch(update).delete(delete),
        )
        .route(
            "/api/v1/books/{id}/cover",
            axum::routing::post(upload_cover)
                .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .route("/api/v1/books/{id}/stats", axum::routing::get(stats))
}
