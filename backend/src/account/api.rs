use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde_json::Value;

use crate::account::types::{
    ChangePasswordRequest, LoginRequest, LogoutRequest, RefreshRequest, RegisterRequest,
    ResetPasswordRequest, SendEmailCodeRequest,
};
use crate::account::{
    codes, login as account_login, recovery as account_recovery, register as account_register,
    session as account_session,
};
use crate::runtime::error::AppError;
use crate::runtime::middleware::auth::AuthUser;
use crate::runtime::state::AppState;
use crate::shared::api;

fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_string()
}

pub async fn send_email_code(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<SendEmailCodeRequest>,
) -> Result<Json<Value>, AppError> {
    let response =
        codes::send_email_code(&state, &request.email, &request.purpose, &client_ip(&headers))
            .await?;
    Ok(api::ok(response))
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let response = account_register::register(&state, request, &client_ip(&headers)).await?;
    Ok(api::created(response))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> Result<Json<Value>, AppError> {
    let response = account_login::login(&state, request, &client_ip(&headers)).await?;
    Ok(api::ok(response))
}

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<Value>, AppError> {
    let response = account_session::refresh(&state, &request.refresh_token).await?;
    Ok(api::ok(response))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LogoutRequest>,
) -> Result<StatusCode, AppError> {
    account_session::logout(&state, &request.refresh_token).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    let response = account_login::me(&state, user.id).await?;
    Ok(api::ok(response))
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    account_login::change_password(&state, user.id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<StatusCode, AppError> {
    account_recovery::reset_password(&state, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/api/v1/auth/email/code", axum::routing::post(send_email_code))
        .route("/api/v1/auth/register", axum::routing::post(register))
        .route("/api/v1/auth/login", axum::routing::post(login))
        .route("/api/v1/auth/refresh", axum::routing::post(refresh))
        .route("/api/v1/auth/logout", axum::routing::post(logout))
        .route("/api/v1/auth/me", axum::routing::get(me))
        .route(
            "/api/v1/auth/password",
            axum::routing::post(change_password),
        )
        .route(
            "/api/v1/auth/password/reset",
            axum::routing::post(reset_password),
        )
}