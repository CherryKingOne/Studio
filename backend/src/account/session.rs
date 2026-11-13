use crate::account::data::{self, User};
use crate::account::tokens;
use crate::account::types::{RefreshResponse, SessionResponse, UserDto};
use crate::runtime::error::AppError;
use crate::runtime::state::AppState;
use crate::shared::auth::encode_token;

pub async fn issue_session(state: &AppState, user: &User) -> Result<SessionResponse, AppError> {
    let access_token = encode_token(
        &state.config.jwt.secret,
        state.config.jwt.ttl_hours,
        user.id,
        &user.username,
        &user.role,
    )?;
    let refresh = tokens::issue(&state.db, user.id, state.config.jwt.refresh_ttl_days).await?;
    Ok(SessionResponse {
        access_token,
        refresh_token: refresh.token,
        token_type: "Bearer",
        expires_in: state.config.jwt.ttl_hours * 3600,
        refresh_expires_in: refresh.expires_in,
        user: UserDto::from(user.clone()),
    })
}

pub async fn refresh(state: &AppState, refresh_token: &str) -> Result<RefreshResponse, AppError> {
    let token = refresh_token.trim();
    if token.is_empty() {
        return Err(AppError::Unauthorized);
    }
    let (user_id, issued) =
        tokens::rotate(&state.db, token, state.config.jwt.refresh_ttl_days).await?;
    let user = data::find_by_id(&state.db, user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;
    let access_token = encode_token(
        &state.config.jwt.secret,
        state.config.jwt.ttl_hours,
        user.id,
        &user.username,
        &user.role,
    )?;
    Ok(RefreshResponse {
        access_token,
        refresh_token: issued.token,
        token_type: "Bearer",
        expires_in: state.config.jwt.ttl_hours * 3600,
        refresh_expires_in: issued.expires_in,
    })
}

pub async fn logout(state: &AppState, refresh_token: &str) -> Result<(), AppError> {
    tokens::revoke(&state.db, refresh_token.trim()).await
}