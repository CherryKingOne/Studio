use std::time::Duration;

use anyhow::Context;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

use crate::config::AppConfig;

pub async fn connect(config: &AppConfig) -> anyhow::Result<MySqlPool> {
    let pool = MySqlPoolOptions::new()
        .max_connections(config.database.max_connections)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&config.database.url)
        .await
        .with_context(|| format!("connect MySQL failed: {}", config.database.url))?;
    Ok(pool)
}

pub async fn migrate(pool: &MySqlPool) -> anyhow::Result<()> {
    crate::common::schema::apply(pool).await
}

pub async fn ping(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}
