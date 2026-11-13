use chrono::NaiveDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FileRecord {
    pub id: u64,
    pub bucket: String,
    pub object_key: String,
    pub file_name: String,
    pub mime: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub kind: String,
    pub duration_ms: Option<u32>,
    pub created_at: NaiveDateTime,
}
