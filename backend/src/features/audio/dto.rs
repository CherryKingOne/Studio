use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct MergeRequest {
    #[serde(default)]
    pub skip_failed: bool,
    pub pause_ms: Option<PauseOptions>,
    pub loudness: Option<LoudnessOptions>,
    pub output: Option<OutputOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PauseOptions {
    pub narration: u32,
    pub dialogue: u32,
    pub chapter_end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoudnessOptions {
    pub i: f32,
    pub tp: f32,
    pub lra: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputOptions {
    pub sample_rate: u32,
    pub channels: u8,
    pub bitrate_kbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeOptions {
    pub skip_failed: bool,
    pub pause_ms: PauseOptions,
    pub loudness: LoudnessOptions,
    pub output: OutputOptions,
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self {
            skip_failed: false,
            pause_ms: PauseOptions {
                narration: 350,
                dialogue: 250,
                chapter_end: 1200,
            },
            loudness: LoudnessOptions {
                i: -16.0,
                tp: -1.5,
                lra: 11.0,
            },
            output: OutputOptions {
                sample_rate: 44_100,
                channels: 1,
                bitrate_kbps: 128,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergePayload {
    pub chapter_id: u64,
    pub options: MergeOptions,
}

#[derive(Debug, Serialize)]
pub struct MergeResponse {
    pub job_id: u64,
}
