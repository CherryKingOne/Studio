use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

use super::handler;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/voices", get(handler::list).post(handler::create))
        .route("/api/v1/voices/sync", post(handler::sync))
        .route("/api/v1/voices/preview", post(handler::preview))
        .route("/api/v1/voices/{id}/sample", get(handler::sample))
}
