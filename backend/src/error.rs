use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::json;
use tracing::error;

use crate::middleware::request_id::current_request_id;

#[derive(Debug, Clone, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("未认证或 Token 无效")]
    Unauthorized,
    #[error("无权限")]
    Forbidden,
    #[error("{resource} 不存在")]
    NotFound { resource: &'static str },
    #[error("参数校验失败")]
    Validation(Vec<FieldError>),
    #[error("{0}")]
    InvalidState(String),
    #[error("{0}")]
    Duplicate(String),
    #[error("{0}")]
    ResourceInUse(String),
    #[error("仅支持 .txt 文件")]
    UnsupportedFileType,
    #[error("文件过大")]
    FileTooLarge,
    #[error("对象存储错误：{0}")]
    Storage(String),
    #[error("TTS 引擎错误：{0}")]
    TtsProvider(String),
    #[error("TTS 引擎限流")]
    TtsRateLimited,
    #[error("音频处理失败：{0}")]
    AudioProcess(String),
    #[error("已有进行中的同类任务")]
    JobAlreadyActive { existing_job_id: u64 },
    #[error("用户名或密码错误")]
    LoginFailed,
    #[error("请求过于频繁")]
    RateLimited,
    #[error("内部错误")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn status_code(&self) -> (StatusCode, u32) {
        match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, 10001),
            AppError::Forbidden => (StatusCode::FORBIDDEN, 10002),
            AppError::LoginFailed => (StatusCode::UNAUTHORIZED, 10003),
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, 10004),
            AppError::NotFound { .. } => (StatusCode::NOT_FOUND, 20001),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, 20002),
            AppError::InvalidState(_) => (StatusCode::CONFLICT, 20003),
            AppError::Duplicate(_) => (StatusCode::CONFLICT, 20004),
            AppError::ResourceInUse(_) => (StatusCode::CONFLICT, 20005),
            AppError::UnsupportedFileType => (StatusCode::UNSUPPORTED_MEDIA_TYPE, 30001),
            AppError::FileTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, 30002),
            AppError::Storage(_) => (StatusCode::BAD_GATEWAY, 30003),
            AppError::TtsProvider(_) => (StatusCode::BAD_GATEWAY, 40001),
            AppError::TtsRateLimited => (StatusCode::TOO_MANY_REQUESTS, 40002),
            AppError::AudioProcess(_) => (StatusCode::INTERNAL_SERVER_ERROR, 40003),
            AppError::JobAlreadyActive { .. } => (StatusCode::CONFLICT, 40004),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50001),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_code();
        let details = match &self {
            AppError::Validation(errors) => Some(json!(errors)),
            AppError::JobAlreadyActive { existing_job_id } => {
                Some(json!({ "existing_job_id": existing_job_id }))
            }
            _ => None,
        };
        let message = if matches!(&self, AppError::Internal(_)) {
            error!(error = %self, "internal error");
            "内部错误".to_string()
        } else {
            self.to_string()
        };

        let body = json!({
            "code": code,
            "message": message,
            "details": details,
            "request_id": current_request_id(),
        });
        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::Internal(anyhow::Error::new(error))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::Internal(anyhow::Error::new(error))
    }
}
