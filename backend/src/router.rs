use std::sync::Arc;

use axum::Router;

use crate::features::{auth, books, chapters, files, health, imports, jobs, segments};
use crate::state::AppState;

pub fn build() -> Router<Arc<AppState>> {
    Router::new()
        .merge(health::routes::routes())
        .merge(auth::routes::routes())
        .merge(books::routes::routes())
        .merge(imports::routes::routes())
        .merge(chapters::routes::routes())
        .merge(segments::routes::routes())
        .merge(jobs::routes::routes())
        .merge(files::routes::routes())
}
