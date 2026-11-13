use std::sync::Arc;

use axum::Router;

use crate::account;
use crate::audio;
use crate::casting::{characters, voices};
use crate::delivery::exports;
use crate::library::{books, chapters, segments};
use crate::runtime::state::AppState;
use crate::speech;
use crate::system::health;
use crate::task_center;
use crate::text_import;

pub fn build() -> Router<Arc<AppState>> {
    Router::new()
        .merge(health::routes())
        .merge(account::routes())
        .merge(books::routes())
        .merge(text_import::http::routes())
        .merge(chapters::routes())
        .merge(segments::routes())
        .merge(speech::routes())
        .merge(audio::routes())
        .merge(exports::routes::routes())
        .merge(characters::routes())
        .merge(voices::routes())
        .merge(task_center::routes::routes())
        .merge(crate::delivery::files::routes::routes())
}
