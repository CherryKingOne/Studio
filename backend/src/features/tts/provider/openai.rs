use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use serde_json::json;

use crate::config::TtsSection;
use crate::error::AppError;
use crate::features::tts::provider::{
    SynthesisRequest, SynthesizedAudio, TtsProvider, VoiceInfo,
};

const OPENAI_VOICES: &[(&str, &str, &str, &[&str])] = &[
    ("alloy", "Alloy（中性）", "neutral", &["balanced", "narration"]),
    ("ash", "Ash（中性）", "neutral", &["clear", "direct"]),
    ("ballad", "Ballad（叙事男声）", "male", &["storytelling", "warm"]),
    ("coral", "Coral（温暖女声）", "female", &["warm", "friendly"]),
    ("echo", "Echo（男声）", "male", &["steady", "calm"]),
    ("fable", "Fable（英伦男声）", "male", &["storytelling", "expressive"]),
    ("nova", "Nova（女声）", "female", &["clear", "lively"]),
    ("onyx", "Onyx（低沉男声）", "male", &["deep", "narration"]),
    ("sage", "Sage（沉稳女声）", "female", &["calm", "narration"]),
    ("shimmer", "Shimmer（柔和女声）", "female", &["soft", "gentle"]),
    ("verse", "Verse（表现力男声）", "male", &["expressive", "dynamic"]),
];

pub struct OpenAiTtsProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiTtsProvider {
    pub fn new(config: &TtsSection) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs.max(5)))
            .build()
            .unwrap_or_default();
        Self {
            client,
            api_key: config.openai_api_key.clone(),
            base_url: config.openai_base_url.trim_end_matches('/').to_string(),
            model: config.openai_model.clone(),
        }
    }
}

#[async_trait]
impl TtsProvider for OpenAiTtsProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn version(&self) -> &'static str {
        "openai-v1"
    }

    async fn list_voices(&self) -> Result<Vec<VoiceInfo>, AppError> {
        Ok(OPENAI_VOICES
            .iter()
            .map(|(id, name, gender, styles)| VoiceInfo {
                provider_voice_id: (*id).to_string(),
                display_name: (*name).to_string(),
                locale: "multi".to_string(),
                gender: (*gender).to_string(),
                styles: styles.iter().map(|style| (*style).to_string()).collect(),
                extra: None,
            })
            .collect())
    }

    async fn synthesize(
        &self,
        request: &SynthesisRequest,
    ) -> Result<SynthesizedAudio, AppError> {
        if self.api_key.trim().is_empty() {
            return Err(AppError::TtsProvider(
                "未配置 TTS_OPENAI_API_KEY，无法调用 OpenAI TTS".to_string(),
            ));
        }

        let speed = (1.0 + request.rate_percent as f32 / 100.0).clamp(0.25, 4.0);
        let body = json!({
            "model": self.model,
            "input": request.text,
            "voice": request.provider_voice_id,
            "response_format": "mp3",
            "speed": speed,
        });

        let response = self
            .client
            .post(format!("{}/audio/speech", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|error| AppError::TtsProvider(error.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let snippet = body.chars().take(300).collect::<String>();
            return Err(AppError::TtsProvider(format!(
                "OpenAI TTS 请求失败: HTTP {status} {snippet}"
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|error| AppError::TtsProvider(error.to_string()))?;
        if bytes.is_empty() {
            return Err(AppError::TtsProvider("OpenAI TTS 返回音频为空".to_string()));
        }

        Ok(SynthesizedAudio {
            bytes: Bytes::from(bytes.to_vec()),
            format: "mp3".to_string(),
        })
    }
}
