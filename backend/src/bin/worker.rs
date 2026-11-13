use std::sync::Arc;

use novelcast::runtime::config::AppConfig;
use novelcast::runtime::state::{AppState, WorkerState};
use novelcast::shared::db;
use novelcast::shared::logging;
use novelcast::shared::storage::Storage;
use novelcast::speech::provider::TtsRegistry;
use novelcast::worker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    logging::init(&config);

    let db = db::connect(&config).await?;
    let storage = Arc::new(Storage::new(&config.rustfs));
    let app = Arc::new(AppState::new(
        Arc::new(config.clone()),
        db,
        storage,
        Arc::new(TtsRegistry::new(&config.tts)),
    ));

    let worker_id = format!(
        "{}-{}-{}",
        std::env::var("HOSTNAME").unwrap_or_else(|_| "worker".to_string()),
        std::process::id(),
        &uuid::Uuid::now_v7().to_string()[..8]
    );
    let state = Arc::new(WorkerState { app, worker_id });

    worker::dispatcher::run(state).await
}
