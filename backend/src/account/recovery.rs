use crate::account::codes;
use crate::account::data;
use crate::account::tokens;
use crate::account::types::{normalize_email, ResetPasswordRequest};
use crate::runtime::error::{AppError, FieldError};
use crate::runtime::state::AppState;
use crate::shared::auth::hash_password;

pub async fn reset_password(state: &AppState, request: ResetPasswordRequest) -> Result<(), AppError> {
    let email = normalize_email(&request.email)?;
    if request.new_password.chars().count() < 8 {
        return Err(validation("new_password", "新密码至少 8 个字符"));
    }
    let code = request.code.trim();
    if code.is_empty() {
        return Err(validation("code", "验证码不能为空"));
    }

    codes::verify_and_consume(&state.db, &email, codes::PURPOSE_RESET, code).await?;

    let user = data::find_by_email(&state.db, &email)
        .await?
        .ok_or(AppError::NotFound { resource: "用户" })?;
    let password_hash = hash_password(&request.new_password)?;
    data::update_password(&state.db, user.id, &password_hash).await?;
    tokens::revoke_all(&state.db, user.id).await?;
    Ok(())
}

fn validation(field: &str, message: &str) -> AppError {
    AppError::Validation(vec![FieldError {
        field: field.to_string(),
        message: message.to_string(),
    }])
}