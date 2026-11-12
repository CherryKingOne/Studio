use std::sync::Arc;

use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;

use crate::common::api;
use crate::error::{AppError, FieldError};
use crate::features::imports::service;
use crate::middleware::auth::AuthUser;
use crate::state::AppState;

pub async fn import(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let mut file_name: Option<String> = None;
    let mut file_bytes = None;
    let mut title = None;
    let mut author = None;

    while let Some(field) = multipart.next_field().await.map_err(|error| {
        AppError::Validation(vec![FieldError {
            field: "file".to_string(),
            message: error.to_string(),
        }])
    })? {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(str::to_string);
                file_bytes = Some(field.bytes().await.map_err(|error| {
                    AppError::Validation(vec![FieldError {
                        field: "file".to_string(),
                        message: error.to_string(),
                    }])
                })?);
            }
            "title" => {
                title = Some(field.text().await.map_err(|error| {
                    AppError::Validation(vec![FieldError {
                        field: "title".to_string(),
                        message: error.to_string(),
                    }])
                })?);
            }
            "author" => {
                author = Some(field.text().await.map_err(|error| {
                    AppError::Validation(vec![FieldError {
                        field: "author".to_string(),
                        message: error.to_string(),
                    }])
                })?);
            }
            _ => {}
        }
    }

    let file_name = file_name.ok_or_else(|| {
        AppError::Validation(vec![FieldError {
            field: "file".to_string(),
            message: "缺少上传文件".to_string(),
        }])
    })?;
    let bytes = file_bytes.ok_or_else(|| {
        AppError::Validation(vec![FieldError {
            field: "file".to_string(),
            message: "文件内容为空".to_string(),
        }])
    })?;

    let response =
        service::create_import(&state, user.id, &file_name, title, author, bytes).await?;
    Ok(api::accepted(response))
}
