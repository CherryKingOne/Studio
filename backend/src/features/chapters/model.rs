use chrono::NaiveDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Chapter {
    pub id: u64,
    pub book_id: u64,
    pub idx: u32,
    pub title: String,
    pub raw_text: String,
    pub char_count: u32,
    pub status: String,
    pub narrator_voice_id: Option<u64>,
    pub audio_file_id: Option<u64>,
    pub audio_duration_ms: Option<u32>,
    pub audio_rev: u32,
    pub tts_done_segments: u32,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChapterRow {
    #[sqlx(flatten)]
    pub chapter: Chapter,
    pub segment_count: u64,
}
