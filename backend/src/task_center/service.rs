use serde_json::{json, Value};
use sqlx::{MySql, Transaction};

use crate::runtime::error::AppError;
use crate::runtime::state::AppState;
use crate::shared::pagination::Page;
use crate::task_center::dto::{
    CancelResponse, JobDetail, JobItemDto, JobListQuery, JobSummary, RetryResponse,
};
use crate::task_center::model::Job;
use crate::task_center::queue;
use crate::task_center::repo::{self, NewJob, NewJobItem};

pub async fn create_in_tx(
    tx: &mut Transaction<'_, MySql>,
    job: &NewJob,
    items: &[NewJobItem],
) -> Result<u64, AppError> {
    queue::enqueue_tx(tx, job, items).await
}

pub async fn list(state: &AppState, query: JobListQuery) -> Result<Page<JobSummary>, AppError> {
    let (jobs, total) = repo::list(&state.db, &query).await?;
    let items = jobs.iter().map(JobSummary::from).collect();
    Ok(Page::new(items, query.page(), query.page_size(), total))
}

pub async fn get_model(state: &AppState, id: u64) -> Result<Job, AppError> {
    repo::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound { resource: "任务" })
}

pub async fn get(state: &AppState, id: u64, include_items: bool) -> Result<JobDetail, AppError> {
    let job = get_model(state, id).await?;
    let items = if include_items {
        repo::items(&state.db, id, 500)
            .await?
            .into_iter()
            .map(JobItemDto::from)
            .collect()
    } else {
        Vec::new()
    };

    Ok(JobDetail {
        job: JobSummary::from(&job),
        items,
        result: job.result.map(|value| value.0),
    })
}

pub async fn cancel(state: &AppState, id: u64) -> Result<CancelResponse, AppError> {
    let job = get_model(state, id).await?;
    match job.status.as_str() {
        "queued" => {
            repo::cancel_queued(&state.db, id).await?;
            Ok(CancelResponse {
                id,
                status: "canceled".to_string(),
                cancel_requested: false,
            })
        }
        "running" => {
            repo::request_cancel(&state.db, id).await?;
            Ok(CancelResponse {
                id,
                status: job.status,
                cancel_requested: true,
            })
        }
        _ => Err(AppError::InvalidState("任务已结束，不能取消".to_string())),
    }
}

pub async fn retry(
    state: &AppState,
    id: u64,
    retry_failed_only: bool,
) -> Result<RetryResponse, AppError> {
    let job = get_model(state, id).await?;
    if !matches!(job.status.as_str(), "failed" | "partial" | "canceled") {
        return Err(AppError::InvalidState(
            "仅失败、部分成功或已取消的任务可重试".to_string(),
        ));
    }

    let items = if retry_failed_only {
        repo::failed_item_ids(&state.db, id)
            .await?
            .into_iter()
            .map(|item_id| NewJobItem {
                item_type: "segment".to_string(),
                item_id,
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let new_job = NewJob {
        job_type: job.job_type.clone(),
        priority: job.priority,
        book_id: job.book_id,
        chapter_id: job.chapter_id,
        created_by: job.created_by,
        payload: job.payload.0.clone(),
        total_items: items.len() as u32,
        max_attempts: job.max_attempts,
    };
    let job_id = queue::enqueue(&state.db, &new_job, &items).await?;
    Ok(RetryResponse {
        job_id,
        source_job_id: id,
    })
}

pub fn is_terminal(status: &str) -> bool {
    matches!(status, "succeeded" | "partial" | "failed" | "canceled")
}

pub fn job_event_json(job: &Job) -> Value {
    json!({
        "job_id": job.id,
        "status": job.status,
        "progress": job.progress,
        "done_items": job.done_items,
        "failed_items": job.failed_items,
        "total_items": job.total_items,
        "current_step": job.current_step,
        "error_message": job.error_message,
        "updated_at": crate::shared::time::iso(job.updated_at),
    })
}
