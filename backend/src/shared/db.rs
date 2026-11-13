use std::time::Duration;

use anyhow::Context;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

use crate::runtime::config::AppConfig;

pub async fn connect(config: &AppConfig) -> anyhow::Result<MySqlPool> {
    let mut last_error = None;

    for attempt in 1..=30 {
        match MySqlPoolOptions::new()
            .max_connections(config.database.max_connections)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&config.database.url)
            .await
        {
            Ok(pool) => {
                tracing::info!(attempt, "connected to MySQL");
                return Ok(pool);
            }
            Err(error) => {
                tracing::warn!(attempt, %error, "connect MySQL failed, retrying");
                last_error = Some(error);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    Err(anyhow::anyhow!(last_error
        .map(|error| error.to_string())
        .unwrap_or_else(|| "connect MySQL failed".to_string())))
    .with_context(|| format!("MySQL not reachable: {}", config.database.url))
}

pub async fn migrate(pool: &MySqlPool) -> anyhow::Result<()> {
    crate::shared::schema::apply(pool).await
}

pub async fn ping(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}
