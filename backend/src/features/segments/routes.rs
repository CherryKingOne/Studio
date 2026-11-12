use std::sync::Arc;

use axum::routing::{get, patch};
use axum::Router;

use crate::state::AppState;

use super::handler;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/chapters/{chapter_id}/segments", get(handler::list))
        .route("/api/v1/segments/batch", patch(handler::batch_update))
        .route("/api/v1/segments/{id}", patch(handler::update))
}
