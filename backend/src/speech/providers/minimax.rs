use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use serde::Deserialize;
use serde_json::json;

use crate::runtime::config::TtsSection;
use crate::runtime::error::AppError;
use crate::speech::providers::{SynthesisRequest, SynthesizedAudio, TtsProvider, VoiceInfo};

pub struct MiniMaxTtsProvider {
    client: reqwest::Client,
    api_key: String,
    group_id: String,
    base_url: String,
    model: String,
}

#[derive(Debug, Deserialize)]
struct MiniMaxResponse {
    #[serde(default)]
    data: Option<MiniMaxData>,
    #[serde(default)]
    base_resp: Option<MiniMaxBaseResp>,
}

#[derive(Debug, Deserialize)]
struct MiniMaxData {
    audio: String,
    #[serde(default)]
    status: i32,
}

#[derive(Debug, Deserialize)]
struct MiniMaxBaseResp {
    #[serde(default)]
    status_code: i64,
    #[serde(default)]
    status_msg: String,
}

impl MiniMaxTtsProvider {
    pub fn new(config: &TtsSection) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs.max(5)))
            .build()
            .unwrap_or_default();
        Self {
            client,
            api_key: config.minimax_api_key.clone(),
            group_id: config.minimax_group_id.clone(),
            base_url: config.minimax_base_url.trim_end_matches('/').to_string(),
            model: config.minimax_model.clone(),
        }
    }
}

#[async_trait]
impl TtsProvider for MiniMaxTtsProvider {
    fn name(&self) -> &'static str {
        "minimax"
    }

    fn version(&self) -> &'static str {
        "minimax-v1"
    }

    async fn list_voices(&self) -> Result<Vec<VoiceInfo>, AppError> {
        // 不内置音色：音色由用户在 MiniMax 平台获取后手动登记。
        Ok(Vec::new())
    }

    async fn synthesize(&self, request: &SynthesisRequest) -> Result<SynthesizedAudio, AppError> {
        if self.api_key.trim().is_empty() || self.group_id.trim().is_empty() {
            return Err(AppError::TtsProvider(
                "未配置 TTS_MINIMAX_API_KEY / TTS_MINIMAX_GROUP_ID".to_string(),
            ));
        }

        let speed = (1.0 + request.rate_percent as f32 / 100.0).clamp(0.5, 2.0);
        let pitch = (request.pitch_hz as f32 / 5.0).clamp(-12.0, 12.0);
        let body = json!({
            "model": self.model,
            "text": request.text,
            "stream": false,
            "language_boost": "auto",
            "output_format": "hex",
            "voice_setting": {
                "voice_id": request.provider_voice_id,
                "speed": speed,
                "vol": 1.0,
                "pitch": pitch,
            },
            "audio_setting": {
                "sample_rate": 32000,
                "bitrate": 128000,
                "format": "mp3",
            }
        });

        let response = self
            .client
            .post(format!(
                "{}/v1/t2a_v2?GroupId={}",
                self.base_url, self.group_id
            ))
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
                "MiniMax TTS 请求失败: HTTP {status} {snippet}"
            )));
        }

        let payload = response
            .json::<MiniMaxResponse>()
            .await
            .map_err(|error| AppError::TtsProvider(error.to_string()))?;
        if let Some(base_resp) = &payload.base_resp {
            if base_resp.status_code != 0 {
                return Err(AppError::TtsProvider(format!(
                    "MiniMax TTS 错误 {}: {}",
                    base_resp.status_code, base_resp.status_msg
                )));
            }
        }
        let data = payload
            .data
            .ok_or_else(|| AppError::TtsProvider("MiniMax 返回缺少 data".to_string()))?;
        if data.status != 2 && data.audio.is_empty() {
            return Err(AppError::TtsProvider("MiniMax 音频生成未完成".to_string()));
        }

        let bytes = hex::decode(&data.audio)
            .map_err(|error| AppError::TtsProvider(format!("MiniMax 音频解码失败: {error}")))?;
        if bytes.is_empty() {
            return Err(AppError::TtsProvider("MiniMax 返回音频为空".to_string()));
        }
        Ok(SynthesizedAudio {
            bytes: Bytes::from(bytes),
            format: "mp3".to_string(),
        })
    }
}
