use std::sync::Arc;

use axum::Router;

use crate::features::{auth, books, files, health};
use crate::state::AppState;

pub fn build() -> Router<Arc<AppState>> {
    Router::new()
        .merge(health::routes::routes())
        .merge(auth::routes::routes())
        .merge(books::routes::routes())
        .merge(files::routes::routes())
}
