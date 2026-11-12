use std::sync::Arc;

use axum::routing::get;
use axum::Router;

use crate::state::AppState;

use super::handler;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/files/{id}", get(handler::download))
}
