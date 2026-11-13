use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::runtime::state::AppState;

use super::catalog_http;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/v1/voices",
            get(catalog_http::list).post(catalog_http::create),
        )
        .route("/api/v1/voices/sync", post(catalog_http::sync))
        .route("/api/v1/voices/preview", post(catalog_http::preview))
        .route("/api/v1/voices/{id}/sample", get(catalog_http::sample))
}
