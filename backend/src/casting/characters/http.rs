use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::runtime::state::AppState;

use super::manage_http;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/v1/books/{book_id}/characters",
            get(manage_http::list).post(manage_http::create),
        )
        .route(
            "/api/v1/characters/{id}",
            axum::routing::patch(manage_http::update).delete(manage_http::delete),
        )
        .route(
            "/api/v1/books/{book_id}/characters/detect",
            post(manage_http::detect),
        )
}
