use std::sync::Arc;
use std::time::Instant;

use sqlx::MySqlPool;

use crate::runtime::config::AppConfig;
use crate::runtime::middleware::rate_limit::RateLimiter;
use crate::shared::mail::Mailer;
use crate::shared::storage::Storage;
use crate::speech::providers::TtsRegistry;

/// Runtime dependencies shared by the HTTP server and background worker.
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: MySqlPool,
    pub storage: Arc<Storage>,
    pub tts: Arc<TtsRegistry>,
    pub mailer: Arc<Mailer>,
    pub started_at: Instant,
    pub login_limiter: RateLimiter,
    pub preview_limiter: RateLimiter,
    pub code_limiter: RateLimiter,
}

impl AppState {
    pub fn new(
        config: Arc<AppConfig>,
        db: MySqlPool,
        storage: Arc<Storage>,
        tts: Arc<TtsRegistry>,
    ) -> Self {
        let mailer = Arc::new(Mailer::new(&config.mail));
        Self {
            config,
            db,
            storage,
            tts,
            mailer,
            started_at: Instant::now(),
            login_limiter: RateLimiter::new(10, std::time::Duration::from_secs(60)),
            preview_limiter: RateLimiter::new(20, std::time::Duration::from_secs(60)),
            code_limiter: RateLimiter::new(10, std::time::Duration::from_secs(60)),
        }
    }
}

pub struct WorkerState {
    pub app: Arc<AppState>,
    pub worker_id: String,
}
