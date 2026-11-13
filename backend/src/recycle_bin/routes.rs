use std::sync::Arc;

use axum::routing::{delete, get, post};
use axum::Router;

use crate::recycle_bin::handler;
use crate::runtime::state::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/v1/recycle-bin",
            get(handler::list).delete(handler::empty),
        )
        .route("/api/v1/recycle-bin/summary", get(handler::summary))
        .route("/api/v1/recycle-bin/restore", post(handler::batch_restore))
        .route("/api/v1/recycle-bin/purge", post(handler::batch_purge))
        .route(
            "/api/v1/recycle-bin/{resource_type}/{id}/restore",
            post(handler::restore),
        )
        .route(
            "/api/v1/recycle-bin/{resource_type}/{id}",
            delete(handler::purge),
        )
}
