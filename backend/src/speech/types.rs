use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ChapterTtsRequest {
    pub mode: Option<String>,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Deserialize)]
pub struct SegmentTtsRequest {
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Serialize)]
pub struct TtsJobResponse {
    pub job_id: u64,
    pub total_candidates: u64,
    pub cached_hits: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TtsPayload {
    pub chapter_id: u64,
    pub segment_ids: Vec<u64>,
    pub force: bool,
}

#[derive(Debug, Serialize)]
pub struct TtsRunResult {
    pub done: u64,
    pub failed: u64,
    pub cached: u64,
}

#[derive(Debug, Serialize)]
pub struct CacheParams<'a> {
    pub emotion: &'a str,
    pub rate_percent: i16,
    pub pitch_hz: i16,
    pub volume_percent: i16,
    pub format: &'a str,
}

impl CacheParams<'_> {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "emotion": self.emotion,
            "rate_percent": self.rate_percent,
            "pitch_hz": self.pitch_hz,
            "volume_percent": self.volume_percent,
            "format": self.format,
        })
    }
}
