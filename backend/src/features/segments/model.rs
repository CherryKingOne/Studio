use chrono::NaiveDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Segment {
    pub id: u64,
    pub chapter_id: u64,
    pub idx: u32,
    pub text: String,
    pub char_count: u32,
    pub seg_type: String,
    pub speaker_id: Option<u64>,
    pub speaker_confidence: Option<f32>,
    pub voice_id: Option<u64>,
    pub emotion: String,
    pub rate_percent: i16,
    pub pitch_hz: i16,
    pub volume_percent: i16,
    pub status: String,
    pub audio_file_id: Option<u64>,
    pub audio_duration_ms: Option<u32>,
    pub cache_key: Option<String>,
    pub error_message: Option<String>,
    pub attempts: u8,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
