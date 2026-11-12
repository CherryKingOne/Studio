use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde_json::Value;

use crate::common::api;
use crate::error::AppError;
use crate::features::auth::dto::{ChangePasswordRequest, LoginRequest};
use crate::features::auth::service;
use crate::middleware::auth::AuthUser;
use crate::state::AppState;

pub async fn login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> Result<Json<Value>, AppError> {
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");

    let response = service::login(&state, request, client_ip).await?;
    Ok(api::ok(response))
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    let response = service::me(&state, user.id).await?;
    Ok(api::ok(response))
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    service::change_password(&state, user.id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}
