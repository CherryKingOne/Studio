use std::sync::Arc;
use std::time::Instant;

use sqlx::MySqlPool;

use crate::common::storage::Storage;
use crate::config::AppConfig;
use crate::middleware::rate_limit::RateLimiter;

pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: MySqlPool,
    pub storage: Arc<Storage>,
    pub started_at: Instant,
    pub login_limiter: RateLimiter,
}

impl AppState {
    pub fn new(config: Arc<AppConfig>, db: MySqlPool, storage: Arc<Storage>) -> Self {
        Self {
            config,
            db,
            storage,
            started_at: Instant::now(),
            login_limiter: RateLimiter::new(10, std::time::Duration::from_secs(60)),
        }
    }
}

pub struct WorkerState {
    pub app: Arc<AppState>,
    pub worker_id: String,
}
