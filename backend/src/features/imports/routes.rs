use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use axum::Router;

use crate::state::AppState;

use super::handler;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/books/import", post(handler::import))
        .layer(DefaultBodyLimit::max(60 * 1024 * 1024))
}
