use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use axum::Json;
use serde_json::Value;

use crate::delivery::exports::dto::CreateExportRequest;
use crate::delivery::exports::service;
use crate::runtime::error::AppError;
use crate::runtime::middleware::auth::AuthUser;
use crate::runtime::state::AppState;
use crate::shared::api;

pub async fn create(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(book_id): Path<u64>,
    Json(request): Json<CreateExportRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let response = service::create_export(&state, book_id, request, user.id).await?;
    Ok(api::accepted(response))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<Json<Value>, AppError> {
    let export = service::get_export(&state, id).await?;
    Ok(api::ok(export))
}

pub async fn download(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<Response, AppError> {
    let (bytes, file_name) = service::download(&state, id).await?;
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/zip"))
        .header(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!(
                "attachment; filename*=UTF-8''{}",
                percent_encode(&file_name)
            ))
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
        )
        .header(
            CONTENT_LENGTH,
            HeaderValue::from_str(&bytes.len().to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("0")),
        )
        .body(Body::from(bytes))
        .map_err(|error| AppError::Internal(anyhow::anyhow!(error)))
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}
