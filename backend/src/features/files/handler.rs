use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use serde::Deserialize;

use crate::error::AppError;
use crate::features::files::service;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    pub download: Option<u8>,
}

pub async fn download(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
    Query(query): Query<DownloadQuery>,
) -> Result<Response, AppError> {
    let file = service::get(&state, id).await?;
    let bytes = state.storage.get(&file.object_key).await?;

    let disposition = if query.download.unwrap_or(0) == 1 {
        "attachment"
    } else {
        "inline"
    };

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            HeaderValue::from_str(&file.mime)
                .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
        )
        .header(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!(
                "{}; filename*=UTF-8''{}",
                disposition,
                percent_encode(&file.file_name)
            ))
            .unwrap_or_else(|_| HeaderValue::from_static("inline")),
        )
        .header(
            CONTENT_LENGTH,
            HeaderValue::from_str(&file.size_bytes.to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("0")),
        )
        .body(Body::from(bytes))
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(response)
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
