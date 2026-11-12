use std::sync::Arc;

use axum::Router;

use crate::features::{
    audio, auth, books, chapters, characters, exports, files, health, imports, jobs, segments, tts,
    voices,
};
use crate::state::AppState;

pub fn build() -> Router<Arc<AppState>> {
    Router::new()
        .merge(health::routes::routes())
        .merge(auth::routes::routes())
        .merge(books::routes::routes())
        .merge(imports::routes::routes())
        .merge(chapters::routes::routes())
        .merge(segments::routes::routes())
        .merge(tts::routes::routes())
        .merge(audio::routes::routes())
        .merge(exports::routes::routes())
        .merge(characters::routes::routes())
        .merge(voices::routes::routes())
        .merge(jobs::routes::routes())
        .merge(files::routes::routes())
}
