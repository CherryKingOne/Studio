use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::types::Json;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Voice {
    pub id: u64,
    pub provider: String,
    pub provider_voice_id: String,
    pub display_name: String,
    pub locale: String,
    pub gender: String,
    pub styles: Option<Json<Value>>,
    pub sample_file_id: Option<u64>,
    pub is_custom: bool,
    pub enabled: bool,
    pub extra: Option<Json<Value>>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
