use std::env;
use std::str::FromStr;

use anyhow::{bail, Context};

const DEV_JWT_SECRET: &str = "dev-only-secret-change-me-please-0123456789abcdef";

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct AppSection {
    pub host: String,
    pub port: u16,
    pub env: String,
    pub log_level: String,
    pub log_format: String,
}

#[derive(Debug, Clone)]
pub struct AdminSection {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseSection {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct JwtSection {
    pub secret: String,
    pub ttl_hours: u64,
}

#[derive(Debug, Clone)]
pub struct RustFsSection {
    pub endpoint: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
}

#[derive(Debug, Clone)]
pub struct TtsSection {
    pub default_provider: String,
    pub default_narrator_voice_id: String,
    pub max_attempts: u8,
    pub timeout_secs: u64,
    pub concurrency: usize,
    pub provider_min_interval_ms: u64,
}

#[derive(Debug, Clone)]
pub struct WorkerSection {
    pub concurrency: usize,
    pub heartbeat_secs: u64,
    pub stale_secs: u64,
}

#[derive(Debug, Clone)]
pub struct AudioSection {
    pub tmp_dir: String,
    pub ffmpeg_path: String,
    pub ffprobe_path: String,
    pub ffmpeg_concurrency: usize,
}

#[derive(Debug, Clone)]
pub struct LimitsSection {
    pub max_import_bytes: usize,
    pub max_image_bytes: usize,
    pub import_max_chars: usize,
}

#[derive(Debug, Clone)]
pub struct CorsSection {
    pub allowed_origins: String,
}

impl AppConfig {
    /// 所有配置统一从 `.env` / 环境变量加载；缺失时使用安全默认值。
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let config = Self {
            app: AppSection {
                host: env_or("APP_HOST", "0.0.0.0"),
                port: env_parse("APP_PORT", 8080)?,
                env: env_or("APP_ENV", "development"),
                log_level: env_or("LOG_LEVEL", "info"),
                log_format: env_or("LOG_FORMAT", "pretty"),
            },
            admin: AdminSection {
                username: env_or("ADMIN_USERNAME", "admin"),
                password: env_or("ADMIN_PASSWORD", ""),
            },
            database: DatabaseSection {
                url: env_required("DATABASE_URL")?,
                max_connections: env_parse("DB_MAX_CONNECTIONS", 10)?,
            },
            jwt: JwtSection {
                secret: env_or("JWT_SECRET", DEV_JWT_SECRET),
                ttl_hours: env_parse("JWT_TTL_HOURS", 24)?,
            },
            rustfs: RustFsSection {
                endpoint: env_required("RUSTFS_ENDPOINT")?,
                region: env_or("RUSTFS_REGION", "us-east-1"),
                access_key: env_required("RUSTFS_ACCESS_KEY")?,
                secret_key: env_required("RUSTFS_SECRET_KEY")?,
                bucket: env_or("RUSTFS_BUCKET", "novelcast"),
            },
            tts: TtsSection {
                default_provider: env_or("TTS_DEFAULT_PROVIDER", "edge"),
                default_narrator_voice_id: env_or(
                    "TTS_DEFAULT_NARRATOR_VOICE_ID",
                    "zh-CN-YunxiNeural",
                ),
                max_attempts: env_parse("TTS_MAX_ATTEMPTS", 3)?,
                timeout_secs: env_parse("TTS_TIMEOUT_SECS", 45)?,
                concurrency: env_parse("TTS_CONCURRENCY", 4)?,
                provider_min_interval_ms: env_parse("TTS_PROVIDER_MIN_INTERVAL_MS", 250)?,
            },
            worker: WorkerSection {
                concurrency: env_parse("WORKER_CONCURRENCY", 2)?,
                heartbeat_secs: env_parse("WORKER_HEARTBEAT_SECS", 10)?,
                stale_secs: env_parse("WORKER_STALE_SECS", 90)?,
            },
            audio: AudioSection {
                tmp_dir: env_or("AUDIO_TMP_DIR", "/tmp/novelcast"),
                ffmpeg_path: env_or("FFMPEG_PATH", "ffmpeg"),
                ffprobe_path: env_or("FFPROBE_PATH", "ffprobe"),
                ffmpeg_concurrency: env_parse("FFMPEG_CONCURRENCY", 2)?,
            },
            limits: LimitsSection {
                max_import_bytes: env_parse("MAX_IMPORT_BYTES", 52_428_800)?,
                max_image_bytes: env_parse("MAX_IMAGE_BYTES", 5_242_880)?,
                import_max_chars: env_parse("IMPORT_MAX_CHARS", 300)?,
            },
            cors: CorsSection {
                allowed_origins: env_or("CORS_ALLOWED_ORIGINS", "http://localhost:5173"),
            },
        };

        config.validate()?;
        Ok(config)
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
        if !(50..=1000).contains(&self.limits.import_max_chars) {
            bail!("IMPORT_MAX_CHARS must be between 50 and 1000");
        }
        Ok(())
    }
}

fn env_value(key: &str) -> Option<String> {
    env::var(key).ok().filter(|value| !value.trim().is_empty())
}

fn env_or(key: &str, default: &str) -> String {
    env_value(key).unwrap_or_else(|| default.to_string())
}

fn env_required(key: &str) -> anyhow::Result<String> {
    env_value(key).with_context(|| format!("{key} is required, check your .env file"))
}

fn env_parse<T>(key: &str, default: T) -> anyhow::Result<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match env_value(key) {
        Some(value) => value
            .parse::<T>()
            .map_err(|error| anyhow::anyhow!("invalid value for {key}: {value}: {error}")),
        None => Ok(default),
    }
}
