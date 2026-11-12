use std::sync::Arc;

use axum::routing::get;
use axum::Router;

use crate::state::AppState;

use super::handler;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/books/{book_id}/chapters", get(handler::list))
        .route(
            "/api/v1/chapters/{id}",
            get(handler::get).patch(handler::update),
        )
        .route(
            "/api/v1/chapters/{id}/resegment",
            axum::routing::post(handler::resegment),
        )
}
