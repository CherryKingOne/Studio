use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

use super::handler;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/auth/login", post(handler::login))
        .route("/api/v1/auth/me", get(handler::me))
        .route("/api/v1/auth/password", post(handler::change_password))
}
