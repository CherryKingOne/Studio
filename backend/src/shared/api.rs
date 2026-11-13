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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn keys(value: &Value) -> Vec<String> {
        value
            .as_object()
            .expect("json object")
            .keys()
            .cloned()
            .collect()
    }

    #[test]
    fn envelope_keeps_documented_field_order() {
        let Json(value) = ok(json!({ "z": 1, "a": 2 }));

        assert_eq!(keys(&value), vec!["code", "message", "data", "request_id"]);
        assert_eq!(keys(&value["data"]), vec!["z", "a"]);
    }

    #[test]
    fn created_keeps_documented_field_order() {
        let (_, Json(value)) = created(json!({ "b": 1, "a": 2 }));

        assert_eq!(keys(&value), vec!["code", "message", "data", "request_id"]);
    }
}
