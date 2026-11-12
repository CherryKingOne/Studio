use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::features::jobs::model::{Job, JobItem};

#[derive(Debug, Deserialize)]
pub struct JobListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub book_id: Option<u64>,
    pub job_type: Option<String>,
    pub status: Option<String>,
}

impl JobListQuery {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn page_size(&self) -> u64 {
        self.page_size.unwrap_or(20).clamp(1, 100)
    }

    pub fn offset(&self) -> u64 {
        (self.page() - 1) * self.page_size()
    }
}

#[derive(Debug, Serialize)]
pub struct JobSummary {
    pub id: u64,
    pub job_type: String,
    pub status: String,
    pub book_id: Option<u64>,
    pub chapter_id: Option<u64>,
    pub progress: u8,
    pub done_items: u32,
    pub failed_items: u32,
    pub total_items: u32,
    pub current_step: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

impl From<&Job> for JobSummary {
    fn from(job: &Job) -> Self {
        Self {
            id: job.id,
            job_type: job.job_type.clone(),
            status: job.status.clone(),
            book_id: job.book_id,
            chapter_id: job.chapter_id,
            progress: job.progress,
            done_items: job.done_items,
            failed_items: job.failed_items,
            total_items: job.total_items,
            current_step: job.current_step.clone(),
            error_message: job.error_message.clone(),
            created_at: crate::common::time::iso(job.created_at),
            started_at: crate::common::time::iso_opt(job.started_at),
            finished_at: crate::common::time::iso_opt(job.finished_at),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JobItemDto {
    pub item_type: String,
    pub item_id: u64,
    pub status: String,
    pub attempts: u8,
    pub error_message: Option<String>,
}

impl From<JobItem> for JobItemDto {
    fn from(item: JobItem) -> Self {
        Self {
            item_type: item.item_type,
            item_id: item.item_id,
            status: item.status,
            attempts: item.attempts,
            error_message: item.error_message,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JobDetail {
    pub job: JobSummary,
    pub items: Vec<JobItemDto>,
    pub result: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct CancelResponse {
    pub id: u64,
    pub status: String,
    pub cancel_requested: bool,
}

#[derive(Debug, Serialize)]
pub struct RetryResponse {
    pub job_id: u64,
    pub source_job_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct RetryRequest {
    #[serde(default)]
    pub retry_failed_only: bool,
}
