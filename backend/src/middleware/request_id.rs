use axum::extract::Request;
use axum::http::header::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

tokio::task_local! {
    static REQUEST_ID: String;
}

pub async fn request_id_middleware(request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| is_valid_request_id(value))
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());

    let mut response = REQUEST_ID
        .scope(request_id.clone(), next.run(request))
        .await;

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

pub fn current_request_id() -> String {
    REQUEST_ID
        .try_with(|value| value.clone())
        .unwrap_or_default()
}

fn is_valid_request_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}
