use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde_json::Value;

use crate::library::chapters::chapter_actions;
use crate::library::chapters::chapter_types::{
    ChapterListQuery, ResegmentRequest, UpdateChapterRequest,
};
use crate::runtime::error::AppError;
use crate::runtime::middleware::auth::AuthUser;
use crate::runtime::state::AppState;
use crate::shared::api;

pub async fn list(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(book_id): Path<u64>,
    Query(query): Query<ChapterListQuery>,
) -> Result<Json<Value>, AppError> {
    let page = chapter_actions::list(&state, book_id, query).await?;
    Ok(api::ok(page))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<Json<Value>, AppError> {
    let chapter = chapter_actions::get(&state, id).await?;
    Ok(api::ok(chapter))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
    Json(request): Json<UpdateChapterRequest>,
) -> Result<Json<Value>, AppError> {
    let chapter = chapter_actions::update(&state, id, request).await?;
    Ok(api::ok(chapter))
}

pub async fn resegment(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
    Json(request): Json<ResegmentRequest>,
) -> Result<Json<Value>, AppError> {
    let response =
        chapter_actions::resegment(&state, id, request.confirm, request.max_chars).await?;
    Ok(api::ok(response))
}

pub fn routes() -> axum::Router<std::sync::Arc<crate::runtime::state::AppState>> {
    axum::Router::new()
        .route("/api/v1/books/{book_id}/chapters", axum::routing::get(list))
        .route(
            "/api/v1/chapters/{id}",
            axum::routing::get(get).patch(update),
        )
        .route(
            "/api/v1/chapters/{id}/resegment",
            axum::routing::post(resegment),
        )
}
