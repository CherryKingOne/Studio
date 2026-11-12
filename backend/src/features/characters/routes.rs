use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

use super::handler;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/v1/books/{book_id}/characters",
            get(handler::list).post(handler::create),
        )
        .route(
            "/api/v1/characters/{id}",
            axum::routing::patch(handler::update).delete(handler::delete),
        )
        .route(
            "/api/v1/books/{book_id}/characters/detect",
            post(handler::detect),
        )
}
