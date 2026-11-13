use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use crate::runtime::error::AppError;
use crate::runtime::state::AppState;
use crate::shared::auth::decode_token;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: u64,
    pub username: String,
    pub role: String,
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = if header.len() >= 7 && header[..7].eq_ignore_ascii_case("bearer ") {
            &header[7..]
        } else {
            return Err(AppError::Unauthorized);
        };

        let claims = decode_token(token.trim(), &state.config.jwt.secret)?;
        Ok(AuthUser {
            id: claims.sub,
            username: claims.username,
            role: claims.role,
        })
    }
}
