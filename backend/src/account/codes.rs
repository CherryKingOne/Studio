use argon2::password_hash::rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;

use crate::account::data;
use crate::account::types::{normalize_email, SendCodeResponse};
use crate::runtime::error::{AppError, FieldError};
use crate::runtime::state::AppState;

pub const CODE_TTL_SECS: u64 = 300;
pub const RESEND_SECS: u64 = 60;
pub const MAX_PER_HOUR: u64 = 5;

pub const PURPOSE_REGISTER: &str = "register";
pub const PURPOSE_RESET: &str = "reset_password";

#[derive(Debug, sqlx::FromRow)]
struct CodeRow {
    id: u64,
    code_hash: String,
    attempts: u8,
    max_attempts: u8,
}

pub struct SentCode {
    pub code: String,
    pub expires_in: u64,
    pub resend_after: u64,
}

fn code_hash(email: &str, purpose: &str, code: &str) -> String {
    hex::encode(Sha256::digest(format!("{email}:{purpose}:{code}")))
}

fn generate_code() -> String {
    format!("{:06}", OsRng.next_u32() % 1_000_000)
}

/// 开发模式（未配置 SMTP）且 APP_ENV=development 时，向响应中附带验证码便于调试。
pub fn dev_code(state: &AppState, code: &str) -> Option<String> {
    if state.mailer.is_dev() && state.config.app.env == "development" {
        Some(code.to_string())
    } else {
        None
    }
}

/// 统一验证码发送入口：purpose=register 要求邮箱未注册，purpose=reset_password 要求邮箱已注册。
pub async fn send_email_code(
    state: &AppState,
    email_raw: &str,
    purpose: &str,
    client_ip: &str,
) -> Result<SendCodeResponse, AppError> {
    let label = match purpose {
        PURPOSE_REGISTER => "注册",
        PURPOSE_RESET => "重置密码",
        _ => return Err(validation("purpose", "只能是 register 或 reset_password")),
    };

    if !state
        .code_limiter
        .check(&format!("email-code:{purpose}:{client_ip}"))
    {
        return Err(AppError::RateLimited);
    }

    let email = normalize_email(email_raw)?;
    let existing = data::find_by_email(&state.db, &email).await?;
    if purpose == PURPOSE_REGISTER && existing.is_some() {
        return Err(AppError::Duplicate("该邮箱已注册".to_string()));
    }
    if purpose == PURPOSE_RESET && existing.is_none() {
        return Err(AppError::NotFound { resource: "用户" });
    }

    let sent = issue(&state.db, &email, purpose).await?;
    state
        .mailer
        .send_verification_code(&email, &sent.code, label, CODE_TTL_SECS)
        .await?;

    Ok(SendCodeResponse {
        expires_in: sent.expires_in,
        resend_after: sent.resend_after,
        dev_code: dev_code(state, &sent.code),
    })
}

pub async fn issue(pool: &MySqlPool, email: &str, purpose: &str) -> Result<SentCode, AppError> {
    let recent = sqlx::query_scalar::<_, u64>(
        "SELECT CAST(COUNT(*) AS UNSIGNED) FROM email_verification_codes \
         WHERE email = ? AND purpose = ? AND created_at > DATE_SUB(UTC_TIMESTAMP(3), INTERVAL ? SECOND)",
    )
    .bind(email)
    .bind(purpose)
    .bind(RESEND_SECS as i64)
    .fetch_one(pool)
    .await?;
    if recent > 0 {
        return Err(AppError::InvalidState(format!(
            "验证码发送过于频繁，请 {RESEND_SECS} 秒后重试"
        )));
    }

    let hourly = sqlx::query_scalar::<_, u64>(
        "SELECT CAST(COUNT(*) AS UNSIGNED) FROM email_verification_codes \
         WHERE email = ? AND purpose = ? AND created_at > DATE_SUB(UTC_TIMESTAMP(3), INTERVAL 1 HOUR)",
    )
    .bind(email)
    .bind(purpose)
    .fetch_one(pool)
    .await?;
    if hourly >= MAX_PER_HOUR {
        return Err(AppError::RateLimited);
    }

    let code = generate_code();
    sqlx::query(
        "INSERT INTO email_verification_codes (email, purpose, code_hash, expires_at) \
         VALUES (?, ?, ?, DATE_ADD(UTC_TIMESTAMP(3), INTERVAL ? SECOND))",
    )
    .bind(email)
    .bind(purpose)
    .bind(code_hash(email, purpose, &code))
    .bind(CODE_TTL_SECS as i64)
    .execute(pool)
    .await?;

    Ok(SentCode {
        code,
        expires_in: CODE_TTL_SECS,
        resend_after: RESEND_SECS,
    })
}

pub async fn verify_and_consume(
    pool: &MySqlPool,
    email: &str,
    purpose: &str,
    code: &str,
) -> Result<(), AppError> {
    let row = sqlx::query_as::<_, CodeRow>(
        "SELECT id, code_hash, attempts, max_attempts FROM email_verification_codes \
         WHERE email = ? AND purpose = ? AND consumed_at IS NULL AND expires_at > UTC_TIMESTAMP(3) \
         ORDER BY id DESC LIMIT 1",
    )
    .bind(email)
    .bind(purpose)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| validation("code", "验证码无效或已过期"))?;

    if row.attempts >= row.max_attempts {
        return Err(validation("code", "验证码尝试次数过多，请重新获取"));
    }

    sqlx::query("UPDATE email_verification_codes SET attempts = attempts + 1 WHERE id = ?")
        .bind(row.id)
        .execute(pool)
        .await?;

    if row.code_hash != code_hash(email, purpose, code) {
        return Err(validation("code", "验证码错误"));
    }

    sqlx::query("UPDATE email_verification_codes SET consumed_at = UTC_TIMESTAMP(3) WHERE id = ?")
        .bind(row.id)
        .execute(pool)
        .await?;
    Ok(())
}

fn validation(field: &str, message: &str) -> AppError {
    AppError::Validation(vec![FieldError {
        field: field.to_string(),
        message: message.to_string(),
    }])
}
