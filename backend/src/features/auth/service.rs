use crate::common::auth::{encode_token, hash_password, verify_password};
use crate::error::{AppError, FieldError};
use crate::features::auth::dto::{ChangePasswordRequest, LoginRequest, LoginResponse, UserDto};
use crate::features::auth::repo;
use crate::state::AppState;

pub async fn bootstrap_admin(state: &AppState) -> anyhow::Result<()> {
    if repo::count(&state.db).await? > 0 {
        return Ok(());
    }

    let password = state.config.admin.password.trim();
    if password.is_empty() {
        anyhow::bail!("users table is empty and ADMIN_PASSWORD is not set");
    }

    let password_hash = hash_password(password)?;
    repo::insert(
        &state.db,
        &state.config.admin.username,
        &password_hash,
        Some("管理员"),
        "admin",
    )
    .await?;
    tracing::info!(username = %state.config.admin.username, "bootstrap admin created");
    Ok(())
}

pub async fn login(
    state: &AppState,
    request: LoginRequest,
    client_ip: &str,
) -> Result<LoginResponse, AppError> {
    if !state.login_limiter.check(&format!("login:{client_ip}")) {
        return Err(AppError::RateLimited);
    }

    if request.username.trim().is_empty() || request.password.is_empty() {
        return Err(AppError::Validation(vec![FieldError {
            field: "username/password".to_string(),
            message: "不能为空".to_string(),
        }]));
    }

    let user = repo::find_by_username(&state.db, request.username.trim())
        .await?
        .ok_or(AppError::LoginFailed)?;

    if !verify_password(&request.password, &user.password_hash)? {
        return Err(AppError::LoginFailed);
    }

    repo::update_last_login(&state.db, user.id).await?;
    let access_token = encode_token(
        &state.config.jwt.secret,
        state.config.jwt.ttl_hours,
        user.id,
        &user.username,
        &user.role,
    )?;

    Ok(LoginResponse {
        access_token,
        token_type: "Bearer",
        expires_in: state.config.jwt.ttl_hours * 3600,
        user: UserDto::from(user),
    })
}

pub async fn me(state: &AppState, user_id: u64) -> Result<UserDto, AppError> {
    let user = repo::find_by_id(&state.db, user_id)
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

    let user = repo::find_by_id(&state.db, user_id)
        .await?
        .ok_or(AppError::NotFound { resource: "用户" })?;

    if !verify_password(&request.old_password, &user.password_hash)? {
        return Err(AppError::LoginFailed);
    }

    let password_hash = hash_password(&request.new_password)?;
    repo::update_password(&state.db, user_id, &password_hash).await?;
    Ok(())
}
