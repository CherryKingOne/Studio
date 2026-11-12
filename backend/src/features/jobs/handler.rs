use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures_util::Stream;
use serde::Deserialize;
use serde_json::Value;

use crate::common::api;
use crate::error::AppError;
use crate::features::jobs::dto::{JobListQuery, RetryRequest};
use crate::features::jobs::service;
use crate::middleware::auth::AuthUser;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct GetJobQuery {
    #[serde(default)]
    pub include_items: bool,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Query(query): Query<JobListQuery>,
) -> Result<Json<Value>, AppError> {
    let page = service::list(&state, query).await?;
    Ok(api::ok(page))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
    Query(query): Query<GetJobQuery>,
) -> Result<Json<Value>, AppError> {
    let detail = service::get(&state, id, query.include_items).await?;
    Ok(api::ok(detail))
}

pub async fn cancel(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<Json<Value>, AppError> {
    let response = service::cancel(&state, id).await?;
    Ok(api::ok(response))
}

pub async fn retry(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
    Json(request): Json<RetryRequest>,
) -> Result<Json<Value>, AppError> {
    let response = service::retry(&state, id, request.retry_failed_only).await?;
    Ok(api::ok(response))
}

pub async fn events(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<u64>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let _ = service::get_model(&state, id).await?;
    let stream_state = state.clone();

    let stream = async_stream::stream! {
        let mut last_snapshot = String::new();
        loop {
            match service::get_model(&stream_state, id).await {
                Ok(job) => {
                    let payload = service::job_event_json(&job);
                    let snapshot = payload.to_string();
                    let terminal = service::is_terminal(&job.status);
                    if snapshot != last_snapshot {
                        let event_name = if terminal { "done" } else { "progress" };
                        yield Ok::<Event, Infallible>(
                            Event::default().event(event_name).data(snapshot.clone()),
                        );
                        last_snapshot = snapshot;
                    }
                    if terminal {
                        break;
                    }
                }
                Err(error) => {
                    yield Ok::<Event, Infallible>(
                        Event::default().event("error").data(error.to_string()),
                    );
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(800)).await;
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
