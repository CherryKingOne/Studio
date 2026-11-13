use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::runtime::state::AppState;

use super::endpoints;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/chapters/{id}/merge", post(endpoints::merge))
        .route(
            "/api/v1/chapters/{id}/audio",
            get(endpoints::chapter_audio).delete(endpoints::delete_chapter_audio),
        )
        .route("/api/v1/segments/{id}/audio", get(endpoints::segment_audio))
}
