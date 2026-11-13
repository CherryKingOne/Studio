use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use serde_json::{json, Value};

use crate::runtime::middleware::request_id::current_request_id;

pub fn ok<T: Serialize>(data: T) -> Json<Value> {
    Json(json!({
        "code": 0,
        "message": "ok",
        "data": data,
        "request_id": current_request_id(),
    }))
}

pub fn created<T: Serialize>(data: T) -> (StatusCode, Json<Value>) {
    (
        StatusCode::CREATED,
        Json(json!({
            "code": 0,
            "message": "created",
            "data": data,
            "request_id": current_request_id(),
        })),
    )
}

pub fn accepted<T: Serialize>(data: T) -> (StatusCode, Json<Value>) {
    (
        StatusCode::ACCEPTED,
        Json(json!({
            "code": 0,
            "message": "accepted",
            "data": data,
            "request_id": current_request_id(),
        })),
    )
}
