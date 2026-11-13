use std::sync::Arc;

use axum::routing::post;
use axum::Router;

use crate::runtime::state::AppState;

use super::synthesize_http;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/v1/chapters/{id}/tts",
            post(synthesize_http::chapter_tts),
        )
        .route(
            "/api/v1/segments/{id}/tts",
            post(synthesize_http::segment_tts),
        )
}
