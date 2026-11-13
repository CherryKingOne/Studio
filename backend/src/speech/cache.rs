use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::types::Json;
use sqlx::{MySql, MySqlPool, Transaction};

pub struct CachedAudio {
    pub file_id: u64,
    pub duration_ms: u32,
}

#[allow(clippy::too_many_arguments)]
pub fn cache_key(
    provider: &str,
    provider_version: &str,
    provider_voice_id: &str,
    text: &str,
    emotion: &str,
    rate_percent: i16,
    pitch_hz: i16,
    volume_percent: i16,
) -> String {
    let input = format!(
        "{provider}|{provider_version}|{provider_voice_id}|{text}|{emotion}|{rate_percent}|{pitch_hz}|{volume_percent}|mp3"
    );
    hex::encode(Sha256::digest(input.as_bytes()))
}

pub fn text_sha256(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

pub async fn get(pool: &MySqlPool, cache_key: &str) -> Result<Option<CachedAudio>, sqlx::Error> {
    let row = sqlx::query_as::<_, (u64, u32)>(
        "SELECT file_id, duration_ms FROM audio_cache WHERE cache_key = ?",
    )
    .bind(cache_key)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(file_id, duration_ms)| CachedAudio {
        file_id,
        duration_ms,
    }))
}

pub async fn touch(pool: &MySqlPool, cache_key: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE audio_cache SET hit_count = hit_count + 1, last_hit_at = UTC_TIMESTAMP(3) WHERE cache_key = ?",
    )
    .bind(cache_key)
    .execute(pool)
    .await?;
    Ok(())
}

pub struct NewCacheEntry {
    pub cache_key: String,
    pub provider: String,
    pub provider_voice_id: String,
    pub text_sha256: String,
    pub params_json: Value,
    pub file_id: u64,
    pub duration_ms: u32,
}

pub async fn put_tx(
    tx: &mut Transaction<'_, MySql>,
    entry: &NewCacheEntry,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audio_cache
            (cache_key, provider, provider_voice_id, text_sha256, params_json, file_id, duration_ms)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            hit_count = hit_count,
            last_hit_at = UTC_TIMESTAMP(3)
        "#,
    )
    .bind(&entry.cache_key)
    .bind(&entry.provider)
    .bind(&entry.provider_voice_id)
    .bind(&entry.text_sha256)
    .bind(Json(entry.params_json.clone()))
    .bind(entry.file_id)
    .bind(entry.duration_ms)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
