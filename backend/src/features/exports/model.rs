use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::types::Json;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ExportRow {
    pub id: u64,
    pub book_id: u64,
    pub export_type: String,
    pub format: String,
    pub status: String,
    pub options: Json<Value>,
    pub total_chapters: u32,
    pub included_chapters: u32,
    pub file_id: Option<u64>,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
    pub finished_at: Option<NaiveDateTime>,
}
