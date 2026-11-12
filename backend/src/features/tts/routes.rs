use std::sync::Arc;

use axum::routing::post;
use axum::Router;

use crate::state::AppState;

use super::handler;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/chapters/{id}/tts", post(handler::chapter_tts))
        .route("/api/v1/segments/{id}/tts", post(handler::segment_tts))
}
