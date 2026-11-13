use argon2::password_hash::rand_core::{OsRng, RngCore};
use chrono::NaiveDateTime;
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;

use crate::runtime::error::AppError;

#[derive(Debug, sqlx::FromRow)]
struct TokenRow {
    id: u64,
    user_id: u64,
    expires_at: NaiveDateTime,
    revoked_at: Option<NaiveDateTime>,
}

pub struct IssuedToken {
    pub token: String,
    pub expires_in: u64,
}

fn token_hash(token: &str) -> String {
    hex::encode(Sha256::digest(token))
}

fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub async fn issue(pool: &MySqlPool, user_id: u64, ttl_days: u64) -> Result<IssuedToken, AppError> {
    let token = generate_token();
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) \
         VALUES (?, ?, DATE_ADD(UTC_TIMESTAMP(3), INTERVAL ? DAY))",
    )
    .bind(user_id)
    .bind(token_hash(&token))
    .bind(ttl_days as i64)
    .execute(pool)
    .await?;
    Ok(IssuedToken {
        token,
        expires_in: ttl_days * 86400,
    })
}

/// 轮换 refresh_token：旧 token 立即吊销并签发新 token。
/// 检测到已吊销 token 被重用时，吊销该用户全部 refresh_token。
pub async fn rotate(
    pool: &MySqlPool,
    token: &str,
    ttl_days: u64,
) -> Result<(u64, IssuedToken), AppError> {
    let row = sqlx::query_as::<_, TokenRow>(
        "SELECT id, user_id, expires_at, revoked_at FROM refresh_tokens WHERE token_hash = ?",
    )
    .bind(token_hash(token))
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if row.revoked_at.is_some() {
        revoke_all(pool, row.user_id).await?;
        return Err(AppError::Unauthorized);
    }
    if row.expires_at <= chrono::Utc::now().naive_utc() {
        return Err(AppError::Unauthorized);
    }

    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = UTC_TIMESTAMP(3), last_used_at = UTC_TIMESTAMP(3) WHERE id = ?",
    )
    .bind(row.id)
    .execute(pool)
    .await?;

    let issued = issue(pool, row.user_id, ttl_days).await?;
    Ok((row.user_id, issued))
}

pub async fn revoke(pool: &MySqlPool, token: &str) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = UTC_TIMESTAMP(3) WHERE token_hash = ? AND revoked_at IS NULL",
    )
    .bind(token_hash(token))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn revoke_all(pool: &MySqlPool, user_id: u64) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = UTC_TIMESTAMP(3) WHERE user_id = ? AND revoked_at IS NULL",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}