use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::Response;
use axum::Json;
use bytes::Bytes;
use serde_json::Value;

use crate::common::api;
use crate::error::AppError;
use crate::features::voices::dto::{
    CreateVoiceRequest, PreviewRequest, SyncRequest, VoiceListQuery,
};
use crate::features::voices::service;
use crate::middleware::auth::AuthUser;
use crate::state::AppState;

pub async fn list(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Query(query): Query<VoiceListQuery>,
) -> Result<Json<Value>, AppError> {
    let page = service::list(&state, query).await?;
    Ok(api::ok(page))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Json(request): Json<CreateVoiceRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let voice = service::create_manual(&state, request).await?;
    Ok(api::created(voice))
}

pub async fn sync(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Json(request): Json<SyncRequest>,
) -> Result<Json<Value>, AppError> {
    let response = service::sync(&state, request.provider).await?;
    Ok(api::ok(response))
}

pub async fn preview(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    headers: HeaderMap,
    Json(request): Json<PreviewRequest>,
) -> Result<Response, AppError> {
    let client_ip = client_ip(&headers);
    if !state.preview_limiter.check(&format!("preview:{client_ip}")) {
        return Err(AppError::RateLimited);
    }
    let bytes = service::preview(&state, request).await?;
    audio_response(bytes)
}

pub async fn sample(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<Response, AppError> {
    let bytes = service::sample(&state, id).await?;
    audio_response(bytes)
}

fn audio_response(bytes: Bytes) -> Result<Response, AppError> {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static("audio/mpeg"))
        .header(
            CONTENT_LENGTH,
            HeaderValue::from_str(&bytes.len().to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("0")),
        )
        .body(Body::from(bytes))
        .map_err(|error| AppError::Internal(anyhow::anyhow!(error)))
}

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
