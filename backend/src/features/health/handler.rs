use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

use crate::common::db;
use crate::state::AppState;

pub async fn index(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "service": "novelcast-server",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "ok",
        "api_base": "/api/v1",
        "health": "/healthz",
        "ready": "/readyz",
        "uptime_secs": state.started_at.elapsed().as_secs(),
    }))
}

pub async fn healthz(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": state.started_at.elapsed().as_secs(),
    }))
}

pub async fn readyz(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    let mysql = match db::ping(&state.db).await {
        Ok(()) => "ok".to_string(),
        Err(error) => format!("error: {error}"),
    };
    let rustfs = match state.storage.health().await {
        Ok(()) => "ok".to_string(),
        Err(error) => format!("error: {error}"),
    };
    let ffmpeg = match ffmpeg_ok(&state.config.audio.ffmpeg_path).await {
        true => "ok".to_string(),
        false => "error: ffmpeg unavailable".to_string(),
    };

    let healthy = mysql == "ok" && rustfs == "ok" && ffmpeg == "ok";
    let status = if healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(json!({
            "status": if healthy { "ok" } else { "degraded" },
            "checks": {
                "mysql": mysql,
                "rustfs": rustfs,
                "ffmpeg": ffmpeg,
            }
        })),
    )
}

async fn ffmpeg_ok(path: &str) -> bool {
    matches!(
        tokio::process::Command::new(path)
            .arg("-version")
            .output()
            .await,
        Ok(output) if output.status.success()
    )
}
