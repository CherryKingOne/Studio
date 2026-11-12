use std::sync::Arc;
use std::time::Instant;

use sqlx::MySqlPool;

use crate::common::storage::Storage;
use crate::config::AppConfig;
use crate::features::tts::provider::TtsRegistry;
use crate::middleware::rate_limit::RateLimiter;

pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: MySqlPool,
    pub storage: Arc<Storage>,
    pub tts: Arc<TtsRegistry>,
    pub started_at: Instant,
    pub login_limiter: RateLimiter,
    pub preview_limiter: RateLimiter,
}

impl AppState {
    pub fn new(
        config: Arc<AppConfig>,
        db: MySqlPool,
        storage: Arc<Storage>,
        tts: Arc<TtsRegistry>,
    ) -> Self {
        Self {
            config,
            db,
            storage,
            tts,
            started_at: Instant::now(),
            login_limiter: RateLimiter::new(10, std::time::Duration::from_secs(60)),
            preview_limiter: RateLimiter::new(20, std::time::Duration::from_secs(60)),
        }
    }
}

pub struct WorkerState {
    pub app: Arc<AppState>,
    pub worker_id: String,
}
