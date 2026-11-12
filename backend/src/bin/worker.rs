use std::sync::Arc;

use novelcast::common::db;
use novelcast::common::logging;
use novelcast::common::storage::Storage;
use novelcast::config::AppConfig;
use novelcast::state::{AppState, WorkerState};
use novelcast::worker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    logging::init(&config);

    let db = db::connect(&config).await?;
    let storage = Arc::new(Storage::new(&config.rustfs));
    let app = Arc::new(AppState::new(Arc::new(config), db, storage));

    let worker_id = format!(
        "{}-{}-{}",
        std::env::var("HOSTNAME").unwrap_or_else(|_| "worker".to_string()),
        std::process::id(),
        &uuid::Uuid::now_v7().to_string()[..8]
    );
    let state = Arc::new(WorkerState { app, worker_id });

    worker::dispatcher::run(state).await
}
