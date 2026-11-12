use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

use super::handler;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/chapters/{id}/merge", post(handler::merge))
        .route(
            "/api/v1/chapters/{id}/audio",
            get(handler::chapter_audio).delete(handler::delete_chapter_audio),
        )
        .route("/api/v1/segments/{id}/audio", get(handler::segment_audio))
}
