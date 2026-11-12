use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::types::Json;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Book {
    pub id: u64,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub language: String,
    pub cover_file_id: Option<u64>,
    pub source_file_id: Option<u64>,
    pub default_narrator_voice_id: Option<u64>,
    pub settings: Json<Value>,
    pub status: String,
    pub total_chapters: u32,
    pub total_segments: u32,
    pub total_duration_ms: u64,
    pub created_by: Option<u64>,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}
