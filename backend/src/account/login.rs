use crate::account::data;
use crate::account::session;
use crate::account::tokens;
use crate::account::types::{
    normalize_email, ChangePasswordRequest, LoginRequest, SessionResponse, UserDto,
};
use crate::runtime::error::{AppError, FieldError};
use crate::runtime::state::AppState;
use crate::shared::auth::{hash_password, verify_password};

pub async fn bootstrap_admin(state: &AppState) -> anyhow::Result<()> {
    if data::count(&state.db).await? > 0 {
        return Ok(());
    }

    let password = state.config.admin.password.trim();
    if password.is_empty() {
        anyhow::bail!("users table is empty and ADMIN_PASSWORD is not set");
    }

    let password_hash = hash_password(password)?;
    data::insert(
        &state.db,
        &state.config.admin.username,
        Some(state.config.admin.email.trim()),
        &password_hash,
        Some("管理员"),
        "admin",
        true,
    )
    .await?;
    tracing::info!(
        username = %state.config.admin.username,
        email = %state.config.admin.email,
        "bootstrap admin created"
    );
    Ok(())
}

pub async fn login(
    state: &AppState,
    request: LoginRequest,
    client_ip: &str,
) -> Result<SessionResponse, AppError> {
    if !state.login_limiter.check(&format!("login:{client_ip}")) {
        return Err(AppError::RateLimited);
    }

    let email = normalize_email(&request.email)?;
    if request.password.is_empty() {
        return Err(AppError::Validation(vec![FieldError {
            field: "password".to_string(),
            message: "不能为空".to_string(),
        }]));
    }

    let user = data::find_by_email(&state.db, &email)
        .await?
        .ok_or(AppError::LoginFailed)?;

    if !verify_password(&request.password, &user.password_hash)? {
        return Err(AppError::LoginFailed);
    }

    data::update_last_login(&state.db, user.id).await?;
    session::issue_session(state, &user).await
}

pub async fn me(state: &AppState, user_id: u64) -> Result<UserDto, AppError> {
    let user = data::find_by_id(&state.db, user_id)
        .await?
        .ok_or(AppError::NotFound { resource: "用户" })?;
    Ok(UserDto::from(user))
}

pub async fn change_password(
    state: &AppState,
    user_id: u64,
    request: ChangePasswordRequest,
) -> Result<(), AppError> {
    if request.new_password.chars().count() < 8 {
        return Err(AppError::Validation(vec![FieldError {
            field: "new_password".to_string(),
            message: "新密码至少 8 个字符".to_string(),
        }]));
    }

    let user = data::find_by_id(&state.db, user_id)
        .await?
        .ok_or(AppError::NotFound { resource: "用户" })?;

    if !verify_password(&request.old_password, &user.password_hash)? {
        return Err(AppError::LoginFailed);
    }

    let password_hash = hash_password(&request.new_password)?;
    data::update_password(&state.db, user_id, &password_hash).await?;
    tokens::revoke_all(&state.db, user_id).await?;
    Ok(())
}
