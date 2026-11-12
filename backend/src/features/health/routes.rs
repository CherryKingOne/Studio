use std::sync::Arc;

use axum::routing::get;
use axum::Router;

use crate::state::AppState;

use super::handler;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handler::index))
        .route("/healthz", get(handler::healthz))
        .route("/readyz", get(handler::readyz))
}
