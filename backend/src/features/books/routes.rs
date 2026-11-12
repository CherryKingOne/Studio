use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

use super::handler;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/books", get(handler::list).post(handler::create))
        .route(
            "/api/v1/books/{id}",
            get(handler::get)
                .patch(handler::update)
                .delete(handler::delete),
        )
        .route(
            "/api/v1/books/{id}/cover",
            post(handler::upload_cover).layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .route("/api/v1/books/{id}/stats", get(handler::stats))
}
