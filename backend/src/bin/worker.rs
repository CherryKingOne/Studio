use std::sync::Arc;
use std::time::Duration;

use novelcast::common::db;
use novelcast::common::logging;
use novelcast::common::signal;
use novelcast::common::storage::Storage;
use novelcast::config::AppConfig;
use novelcast::state::{AppState, WorkerState};

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
    let worker = WorkerState { app, worker_id };

    tracing::info!(
        worker_id = %worker.worker_id,
        "novelcast worker started; M0 dispatcher is idle"
    );

    let shutdown = signal::shutdown_signal();
    tokio::pin!(shutdown);
    let mut heartbeat = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = &mut shutdown => break,
            _ = heartbeat.tick() => {
                tracing::debug!("worker idle: no job handlers registered yet");
            }
        }
    }

    tracing::info!("novelcast worker stopped");
    Ok(())
}
