use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{bail, Context};
use figment::providers::{Format, Toml};
use figment::Figment;
use serde::Deserialize;

const DEV_JWT_SECRET: &str = "dev-only-secret-change-me-please-0123456789abcdef";

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app: AppSection,
    pub admin: AdminSection,
    pub database: DatabaseSection,
    pub jwt: JwtSection,
    pub rustfs: RustFsSection,
    pub tts: TtsSection,
    pub worker: WorkerSection,
    pub audio: AudioSection,
    pub limits: LimitsSection,
    pub cors: CorsSection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppSection {
    pub host: String,
    pub port: u16,
    pub env: String,
    pub log_level: String,
    pub log_format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminSection {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSection {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtSection {
    pub secret: String,
    pub ttl_hours: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RustFsSection {
    pub endpoint: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TtsSection {
    pub default_provider: String,
    pub default_narrator_voice_id: String,
    pub max_attempts: u8,
    pub timeout_secs: u64,
    pub concurrency: usize,
    pub provider_min_interval_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerSection {
    pub concurrency: usize,
    pub heartbeat_secs: u64,
    pub stale_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AudioSection {
    pub tmp_dir: String,
    pub ffmpeg_path: String,
    pub ffprobe_path: String,
    pub ffmpeg_concurrency: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LimitsSection {
    pub max_import_bytes: usize,
    pub max_image_bytes: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorsSection {
    pub allowed_origins: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let env_name = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config");

        let mut figment = Figment::new().merge(Toml::file(config_dir.join("default.toml")));
        let env_file = config_dir.join(format!("{env_name}.toml"));
        if env_file.exists() {
            figment = figment.merge(Toml::file(env_file));
        }

        let mut config: Self = figment.extract().context("load configuration failed")?;
        config.app.env = env_name;
        config.apply_env()?;
        config.validate()?;
        Ok(config)
    }

    fn apply_env(&mut self) -> anyhow::Result<()> {
        set_string(&mut self.app.host, "APP_HOST");
        set_parse(&mut self.app.port, "APP_PORT")?;
        set_string(&mut self.app.log_level, "LOG_LEVEL");
        set_string(&mut self.app.log_format, "LOG_FORMAT");

        set_string(&mut self.admin.username, "ADMIN_USERNAME");
        set_string(&mut self.admin.password, "ADMIN_PASSWORD");

        set_string(&mut self.database.url, "DATABASE_URL");
        set_parse(&mut self.database.max_connections, "DB_MAX_CONNECTIONS")?;

        set_string(&mut self.jwt.secret, "JWT_SECRET");
        set_parse(&mut self.jwt.ttl_hours, "JWT_TTL_HOURS")?;

        set_string(&mut self.rustfs.endpoint, "RUSTFS_ENDPOINT");
        set_string(&mut self.rustfs.region, "RUSTFS_REGION");
        set_string(&mut self.rustfs.access_key, "RUSTFS_ACCESS_KEY");
        set_string(&mut self.rustfs.secret_key, "RUSTFS_SECRET_KEY");
        set_string(&mut self.rustfs.bucket, "RUSTFS_BUCKET");

        set_string(&mut self.tts.default_provider, "TTS_DEFAULT_PROVIDER");
        set_string(
            &mut self.tts.default_narrator_voice_id,
            "TTS_DEFAULT_NARRATOR_VOICE_ID",
        );
        set_parse(&mut self.tts.max_attempts, "TTS_MAX_ATTEMPTS")?;
        set_parse(&mut self.tts.timeout_secs, "TTS_TIMEOUT_SECS")?;
        set_parse(&mut self.tts.concurrency, "TTS_CONCURRENCY")?;
        set_parse(
            &mut self.tts.provider_min_interval_ms,
            "TTS_PROVIDER_MIN_INTERVAL_MS",
        )?;

        set_parse(&mut self.worker.concurrency, "WORKER_CONCURRENCY")?;
        set_parse(&mut self.worker.heartbeat_secs, "WORKER_HEARTBEAT_SECS")?;
        set_parse(&mut self.worker.stale_secs, "WORKER_STALE_SECS")?;

        set_string(&mut self.audio.tmp_dir, "AUDIO_TMP_DIR");
        set_string(&mut self.audio.ffmpeg_path, "FFMPEG_PATH");
        set_string(&mut self.audio.ffprobe_path, "FFPROBE_PATH");
        set_parse(&mut self.audio.ffmpeg_concurrency, "FFMPEG_CONCURRENCY")?;

        set_parse(&mut self.limits.max_import_bytes, "MAX_IMPORT_BYTES")?;
        set_parse(&mut self.limits.max_image_bytes, "MAX_IMAGE_BYTES")?;

        set_string(&mut self.cors.allowed_origins, "CORS_ALLOWED_ORIGINS");

        Ok(())
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.jwt.secret.len() < 32 {
            bail!("JWT_SECRET must be at least 32 bytes");
        }
        if self.app.env == "production" {
            if self.jwt.secret == DEV_JWT_SECRET {
                bail!("JWT_SECRET must be overridden in production");
            }
            if self.cors.allowed_origins.trim().is_empty() {
                bail!("CORS_ALLOWED_ORIGINS must be set in production");
            }
        }
        if self.app.port == 0 {
            bail!("APP_PORT must not be 0");
        }
        if !(1..=16).contains(&self.tts.concurrency) {
            bail!("TTS_CONCURRENCY must be between 1 and 16");
        }
        if !(1..=8).contains(&self.worker.concurrency) {
            bail!("WORKER_CONCURRENCY must be between 1 and 8");
        }
        if !(1..=8).contains(&self.audio.ffmpeg_concurrency) {
            bail!("FFMPEG_CONCURRENCY must be between 1 and 8");
        }
        Ok(())
    }
}

fn env_value(key: &str) -> Option<String> {
    env::var(key).ok().filter(|value| !value.trim().is_empty())
}

fn set_string(target: &mut String, key: &str) {
    if let Some(value) = env_value(key) {
        *target = value;
    }
}

fn set_parse<T>(target: &mut T, key: &str) -> anyhow::Result<()>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    if let Some(value) = env_value(key) {
        let parsed = value
            .parse::<T>()
            .map_err(|error| anyhow::anyhow!("invalid value for {key}: {value}: {error}"))?;
        *target = parsed;
    }
    Ok(())
}
