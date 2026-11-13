use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::runtime::state::AppState;

use super::handler;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/books/{id}/exports", post(handler::create))
        .route("/api/v1/exports/{id}", get(handler::get))
        .route("/api/v1/exports/{id}/download", get(handler::download))
}
