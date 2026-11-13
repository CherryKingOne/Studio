use serde::{Deserialize, Serialize};

use crate::account::data::User;
use crate::runtime::error::{AppError, FieldError};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct SendCodeRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub code: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: u64,
    pub username: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub role: String,
    pub last_login_at: Option<String>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            email_verified: user.email_verified_at.is_some(),
            display_name: user.display_name,
            role: user.role,
            last_login_at: crate::shared::time::iso_opt(user.last_login_at),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    pub refresh_expires_in: u64,
    pub user: UserDto,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    pub refresh_expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct SendCodeResponse {
    pub expires_in: u64,
    pub resend_after: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_code: Option<String>,
}

pub fn normalize_email(raw: &str) -> Result<String, AppError> {
    let email = raw.trim().to_ascii_lowercase();
    if !is_valid_email(&email) {
        return Err(AppError::Validation(vec![FieldError {
            field: "email".to_string(),
            message: "邮箱格式不正确".to_string(),
        }]));
    }
    Ok(email)
}

fn is_valid_email(email: &str) -> bool {
    if email.is_empty() || email.chars().count() > 255 || email.contains(char::is_whitespace) {
        return false;
    }
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_normalization() {
        assert_eq!(normalize_email("  User@Example.COM ").unwrap(), "user@example.com");
        assert!(normalize_email("plain").is_err());
        assert!(normalize_email("a@b").is_err());
        assert!(normalize_email("a@b..com").is_err());
        assert!(normalize_email("a b@c.com").is_err());
        assert!(normalize_email("").is_err());
    }
}