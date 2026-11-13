use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use axum::Json;
use bytes::Bytes;
use serde_json::Value;

use crate::audio::merge_chapter;
use crate::audio::types::MergeRequest;
use crate::runtime::error::AppError;
use crate::runtime::middleware::auth::AuthUser;
use crate::runtime::state::AppState;
use crate::shared::api;

pub async fn merge(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(chapter_id): Path<u64>,
    Json(request): Json<MergeRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let job_id = merge_chapter::create_merge_job(&state, chapter_id, request, user.id).await?;
    Ok(api::accepted(serde_json::json!({ "job_id": job_id })))
}

pub async fn chapter_audio(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(chapter_id): Path<u64>,
) -> Result<Response, AppError> {
    let (bytes, file_name) = merge_chapter::chapter_audio(&state, chapter_id).await?;
    audio_response(bytes, &file_name, "audio/mpeg")
}

pub async fn delete_chapter_audio(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(chapter_id): Path<u64>,
) -> Result<StatusCode, AppError> {
    merge_chapter::delete_chapter_audio(&state, chapter_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn segment_audio(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(segment_id): Path<u64>,
) -> Result<Response, AppError> {
    let (bytes, file_name) = merge_chapter::segment_audio(&state, segment_id).await?;
    audio_response(bytes, &file_name, "audio/mpeg")
}

fn audio_response(bytes: Bytes, file_name: &str, mime: &str) -> Result<Response, AppError> {
    Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            HeaderValue::from_str(mime)
                .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
        )
        .header(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!(
                "inline; filename*=UTF-8''{}",
                percent_encode(file_name)
            ))
            .unwrap_or_else(|_| HeaderValue::from_static("inline")),
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
