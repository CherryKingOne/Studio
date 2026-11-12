use serde::{Deserialize, Serialize};

use crate::features::segments::model::Segment;

#[derive(Debug, Deserialize)]
pub struct SegmentListQuery {
    pub seg_type: Option<String>,
    pub status: Option<String>,
    pub speaker_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct SegmentDto {
    pub id: u64,
    pub index: u32,
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
    pub audio_duration_ms: Option<u32>,
    pub error_message: Option<String>,
}

impl From<Segment> for SegmentDto {
    fn from(segment: Segment) -> Self {
        Self {
            id: segment.id,
            index: segment.idx,
            text: segment.text,
            char_count: segment.char_count,
            seg_type: segment.seg_type,
            speaker_id: segment.speaker_id,
            speaker_confidence: segment.speaker_confidence,
            voice_id: segment.voice_id,
            emotion: segment.emotion,
            rate_percent: segment.rate_percent,
            pitch_hz: segment.pitch_hz,
            volume_percent: segment.volume_percent,
            status: segment.status,
            audio_duration_ms: segment.audio_duration_ms,
            error_message: segment.error_message,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSegmentRequest {
    pub text: Option<String>,
    pub seg_type: Option<String>,
    pub speaker_id: Option<u64>,
    pub voice_id: Option<u64>,
    pub emotion: Option<String>,
    pub rate_percent: Option<i16>,
    pub pitch_hz: Option<i16>,
    pub volume_percent: Option<i16>,
}

#[derive(Debug, Deserialize)]
pub struct BatchSegmentItem {
    pub id: u64,
    #[serde(flatten)]
    pub patch: UpdateSegmentRequest,
}

#[derive(Debug, Deserialize)]
pub struct BatchUpdateRequest {
    pub items: Vec<BatchSegmentItem>,
}

#[derive(Debug, Serialize)]
pub struct BatchUpdateResponse {
    pub updated: u64,
}

#[derive(Debug, Serialize)]
pub struct SegmentListResponse {
    pub items: Vec<SegmentDto>,
    pub total: u64,
}
