use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use axum::Router;

use crate::runtime::state::AppState;

use super::upload_http;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/books/import", post(upload_http::import))
        .layer(DefaultBodyLimit::max(60 * 1024 * 1024))
}
