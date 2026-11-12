use serde::{Deserialize, Serialize};

use crate::features::voices::model::Voice;

#[derive(Debug, Deserialize)]
pub struct VoiceListQuery {
    pub provider: Option<String>,
    pub locale: Option<String>,
    pub gender: Option<String>,
    pub keyword: Option<String>,
    pub enabled: Option<bool>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

impl VoiceListQuery {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn page_size(&self) -> u64 {
        self.page_size.unwrap_or(50).clamp(1, 200)
    }

    pub fn offset(&self) -> u64 {
        (self.page() - 1) * self.page_size()
    }
}

#[derive(Debug, Serialize)]
pub struct VoiceDto {
    pub id: u64,
    pub provider: String,
    pub provider_voice_id: String,
    pub display_name: String,
    pub locale: String,
    pub gender: String,
    pub styles: Vec<String>,
    pub sample_url: String,
    pub enabled: bool,
}

impl From<Voice> for VoiceDto {
    fn from(voice: Voice) -> Self {
        let styles = voice
            .styles
            .as_ref()
            .and_then(|styles| styles.0.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        Self {
            id: voice.id,
            sample_url: format!("/api/v1/voices/{}/sample", voice.id),
            provider: voice.provider,
            provider_voice_id: voice.provider_voice_id,
            display_name: voice.display_name,
            locale: voice.locale,
            gender: voice.gender,
            styles,
            enabled: voice.enabled,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub provider: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub provider: String,
    pub synced: u64,
}

#[derive(Debug, Deserialize)]
pub struct PreviewRequest {
    pub voice_id: u64,
    pub text: Option<String>,
    pub rate_percent: Option<i16>,
    pub pitch_hz: Option<i16>,
    pub volume_percent: Option<i16>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedVoiceDto {
    pub voice_id: Option<u64>,
    pub provider: String,
    pub provider_voice_id: Option<String>,
    pub display_name: Option<String>,
    pub source: String,
}
