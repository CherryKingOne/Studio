use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use novelcast::account::service;
use novelcast::runtime::config::AppConfig;
use novelcast::runtime::server;
use novelcast::runtime::state::AppState;
use novelcast::shared::db;
use novelcast::shared::logging;
use novelcast::shared::signal;
use novelcast::shared::storage::Storage;
use novelcast::speech::provider::TtsRegistry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    logging::init(&config);

    let db = db::connect(&config).await?;
    db::migrate(&db).await?;

    let storage = Arc::new(Storage::new(&config.rustfs));
    let mut bucket_ready = false;
    let mut last_storage_error = None;
    for attempt in 1..=30 {
        match storage.ensure_bucket().await {
            Ok(()) => {
                bucket_ready = true;
                break;
            }
            Err(error) => {
                tracing::warn!(attempt, %error, "object storage not ready, retrying");
                last_storage_error = Some(error);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
    if !bucket_ready {
        return Err(anyhow::anyhow!(last_storage_error
            .map(|error| error.to_string())
            .unwrap_or_else(|| "object storage not ready".to_string())));
    }

    let state = Arc::new(AppState::new(
        Arc::new(config.clone()),
        db,
        storage,
        Arc::new(TtsRegistry::new(&config.tts)),
    ));
    service::bootstrap_admin(&state).await?;

    let addr: SocketAddr = format!("{}:{}", config.app.host, config.app.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!(%addr, "novelcast server listening");
    axum::serve(listener, server::build(state))
        .with_graceful_shutdown(signal::shutdown_signal())
        .await?;
    Ok(())
}
