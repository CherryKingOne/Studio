use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::types::Json;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Job {
    pub id: u64,
    pub job_type: String,
    pub status: String,
    pub priority: u8,
    pub book_id: Option<u64>,
    pub chapter_id: Option<u64>,
    pub created_by: Option<u64>,
    pub payload: Json<Value>,
    pub result: Option<Json<Value>>,
    pub total_items: u32,
    pub done_items: u32,
    pub failed_items: u32,
    pub progress: u8,
    pub current_step: Option<String>,
    pub error_message: Option<String>,
    pub attempts: u8,
    pub max_attempts: u8,
    pub cancel_requested: bool,
    pub worker_id: Option<String>,
    pub scheduled_at: NaiveDateTime,
    pub heartbeat_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct JobItem {
    pub id: u64,
    pub job_id: u64,
    pub item_type: String,
    pub item_id: u64,
    pub status: String,
    pub attempts: u8,
    pub error_message: Option<String>,
    pub updated_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}
