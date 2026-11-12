use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::http::{header, HeaderValue, Method};
use axum::middleware::from_fn;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::error::AppError;
use crate::middleware::request_id::request_id_middleware;
use crate::router;
use crate::state::AppState;

pub fn build(state: Arc<AppState>) -> Router {
    let cors = cors_layer(&state);

    Router::new()
        .merge(router::build())
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(from_fn(request_id_middleware))
        .with_state(state)
}

async fn not_found() -> AppError {
    AppError::NotFound { resource: "路由" }
}

fn cors_layer(state: &AppState) -> CorsLayer {
    let origins = state.config.cors.allowed_origins.trim();

    if origins == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let allowed_origins: Vec<HeaderValue> = origins
            .split(',')
            .filter_map(|origin| HeaderValue::from_str(origin.trim()).ok())
            .collect();

        CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
    }
}
