use argon2::password_hash::rand_core::{OsRng, RngCore};

use crate::account::codes;
use crate::account::data;
use crate::account::session;
use crate::account::types::{normalize_email, RegisterRequest, SessionResponse};
use crate::runtime::error::{AppError, FieldError};
use crate::runtime::state::AppState;
use crate::shared::auth::hash_password;

pub async fn register(
    state: &AppState,
    request: RegisterRequest,
    client_ip: &str,
) -> Result<SessionResponse, AppError> {
    if !state.login_limiter.check(&format!("register:{client_ip}")) {
        return Err(AppError::RateLimited);
    }

    let email = normalize_email(&request.email)?;
    if request.password.chars().count() < 8 {
        return Err(validation("password", "密码至少 8 个字符"));
    }
    let code = request.code.trim();
    if code.is_empty() {
        return Err(validation("code", "验证码不能为空"));
    }

    codes::verify_and_consume(&state.db, &email, codes::PURPOSE_REGISTER, code).await?;

    if data::find_by_email(&state.db, &email).await?.is_some() {
        return Err(AppError::Duplicate("该邮箱已注册".to_string()));
    }

    let username = derive_username(state, &email).await?;
    let password_hash = hash_password(&request.password)?;
    let id = data::insert(
        &state.db,
        &username,
        Some(&email),
        &password_hash,
        None,
        "user",
        true,
    )
    .await?;
    let user = data::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound { resource: "用户" })?;
    session::issue_session(state, &user).await
}

async fn derive_username(state: &AppState, email: &str) -> Result<String, AppError> {
    let local = email.split('@').next().unwrap_or("user");
    let base: String = local
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(56)
        .collect();
    let base = if base.is_empty() {
        "user".to_string()
    } else {
        base
    };

    if data::find_by_username(&state.db, &base).await?.is_none() {
        return Ok(base);
    }
    for _ in 0..5 {
        let candidate = format!("{base}_{:04x}", OsRng.next_u32() % 0x10000);
        if data::find_by_username(&state.db, &candidate).await?.is_none() {
            return Ok(candidate);
        }
    }
    Err(AppError::Internal(anyhow::anyhow!("无法生成唯一用户名")))
}

fn validation(field: &str, message: &str) -> AppError {
    AppError::Validation(vec![FieldError {
        field: field.to_string(),
        message: message.to_string(),
    }])
}