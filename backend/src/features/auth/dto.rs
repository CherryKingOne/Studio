use serde::{Deserialize, Serialize};

use crate::features::auth::model::User;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: u64,
    pub username: String,
    pub display_name: Option<String>,
    pub role: String,
    pub last_login_at: Option<String>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            role: user.role,
            last_login_at: crate::common::time::iso_opt(user.last_login_at),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    pub user: UserDto,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}
