use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::runtime::state::AppState;

use super::handler;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/jobs", get(handler::list))
        .route("/api/v1/jobs/{id}", get(handler::get))
        .route("/api/v1/jobs/{id}/cancel", post(handler::cancel))
        .route("/api/v1/jobs/{id}/retry", post(handler::retry))
        .route("/api/v1/jobs/{id}/events", get(handler::events))
}
