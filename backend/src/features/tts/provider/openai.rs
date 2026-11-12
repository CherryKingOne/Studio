use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use serde_json::json;

use crate::error::AppError;
use crate::features::tts::provider::{SynthesisRequest, SynthesizedAudio, TtsProvider, VoiceInfo};

/// OpenAI 兼容的 /audio/speech 接口，OpenAI 与阶跃星辰共用。
pub struct OpenAiCompatibleProvider {
    client: reqwest::Client,
    provider_name: &'static str,
    version: &'static str,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        provider_name: &'static str,
        version: &'static str,
        api_key: String,
        base_url: String,
        model: String,
        timeout_secs: u64,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs.max(5)))
            .build()
            .unwrap_or_default();
        Self {
            client,
            provider_name,
            version,
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
        }
    }
}

#[async_trait]
impl TtsProvider for OpenAiCompatibleProvider {
    fn name(&self) -> &'static str {
        self.provider_name
    }

    fn version(&self) -> &'static str {
        self.version
    }

    async fn list_voices(&self) -> Result<Vec<VoiceInfo>, AppError> {
        // 不内置音色：音色由用户在平台获取后手动登记。
        Ok(Vec::new())
    }

    async fn synthesize(&self, request: &SynthesisRequest) -> Result<SynthesizedAudio, AppError> {
        if self.api_key.trim().is_empty() {
            return Err(AppError::TtsProvider(format!(
                "未配置 {} 的 API Key，无法合成",
                self.provider_name
            )));
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
                "{} TTS 请求失败: HTTP {status} {snippet}",
                self.provider_name
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|error| AppError::TtsProvider(error.to_string()))?;
        if bytes.is_empty() {
            return Err(AppError::TtsProvider("TTS 返回音频为空".to_string()));
        }

        Ok(SynthesizedAudio {
            bytes: Bytes::from(bytes.to_vec()),
            format: "mp3".to_string(),
        })
    }
}
