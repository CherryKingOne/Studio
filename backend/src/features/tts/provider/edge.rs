use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::Value;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;

use crate::error::AppError;
use crate::features::tts::provider::{
    SynthesisRequest, SynthesizedAudio, TtsProvider, VoiceInfo,
};

const EDGE_TRUSTED_CLIENT_TOKEN: &str = "6A5AA1D4EAFF4E9FB37E23D68491D6F4";
const EDGE_VOICES_URL: &str = "https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list";
const EDGE_WS_URL: &str = "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1";
const EDGE_OUTPUT_FORMAT: &str = "audio-24khz-48kbitrate-mono-mp3";

pub struct EdgeProvider {
    client: reqwest::Client,
    timeout: Duration,
}

#[derive(Debug, Deserialize)]
struct EdgeVoice {
    #[serde(rename = "ShortName")]
    short_name: String,
    #[serde(rename = "Locale")]
    locale: String,
    #[serde(rename = "Gender")]
    gender: String,
    #[serde(rename = "FriendlyName")]
    friendly_name: Option<String>,
    #[serde(rename = "VoiceTag")]
    voice_tag: Option<Value>,
}

impl EdgeProvider {
    pub fn new(timeout_secs: u64) -> Self {
        let timeout = Duration::from_secs(timeout_secs.max(5));
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_default();
        Self { client, timeout }
    }
}

#[async_trait]
impl TtsProvider for EdgeProvider {
    fn name(&self) -> &'static str {
        "edge"
    }

    fn version(&self) -> &'static str {
        "edge-v1"
    }

    async fn list_voices(&self) -> Result<Vec<VoiceInfo>, AppError> {
        let response = self
            .client
            .get(EDGE_VOICES_URL)
            .query(&[("trustedclienttoken", EDGE_TRUSTED_CLIENT_TOKEN)])
            .send()
            .await
            .map_err(|error| AppError::TtsProvider(error.to_string()))?;

        if !response.status().is_success() {
            return Err(AppError::TtsProvider(format!(
                "Edge 音色列表请求失败: HTTP {}",
                response.status()
            )));
        }

        let voices = response
            .json::<Vec<EdgeVoice>>()
            .await
            .map_err(|error| AppError::TtsProvider(error.to_string()))?;

        Ok(voices
            .into_iter()
            .map(|voice| {
                let styles = voice
                    .voice_tag
                    .as_ref()
                    .and_then(|tag| tag.get("VoicePersonalities"))
                    .and_then(Value::as_array)
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default();
                VoiceInfo {
                    display_name: voice
                        .friendly_name
                        .unwrap_or_else(|| voice.short_name.clone()),
                    provider_voice_id: voice.short_name,
                    locale: voice.locale,
                    gender: voice.gender.to_ascii_lowercase(),
                    styles,
                    extra: voice.voice_tag,
                }
            })
            .collect())
    }

    async fn synthesize(
        &self,
        request: &SynthesisRequest,
    ) -> Result<SynthesizedAudio, AppError> {
        let url = format!(
            "{EDGE_WS_URL}?TrustedClientToken={EDGE_TRUSTED_CLIENT_TOKEN}&ConnectionId={}",
            uuid::Uuid::new_v4().simple()
        );
        let mut ws_request = url
            .into_client_request()
            .map_err(|error| AppError::TtsProvider(error.to_string()))?;
        ws_request.headers_mut().insert(
            "Origin",
            HeaderValue::from_static("chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold"),
        );
        ws_request.headers_mut().insert(
            "User-Agent",
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36",
            ),
        );

        let (mut socket, _) = tokio::time::timeout(
            Duration::from_secs(10),
            tokio_tungstenite::connect_async(ws_request),
        )
        .await
        .map_err(|_| AppError::TtsProvider("Edge 连接超时".to_string()))?
        .map_err(|error| AppError::TtsProvider(error.to_string()))?;

        let timestamp = chrono::Utc::now().timestamp_millis().to_string();
        let config = format!(
            "X-Timestamp: {timestamp}\r\nContent-Type: application/json; charset=utf-8\r\nPath: speech.config\r\n\r\n{{\"context\":{{\"synthesis\":{{\"audio\":{{\"metadataoptions\":{{\"sentenceBoundaryEnabled\":false,\"wordBoundaryEnabled\":false}},\"outputFormat\":\"{EDGE_OUTPUT_FORMAT}\"}}}}}}}}"
        );
        let request_id = uuid::Uuid::new_v4().simple().to_string();
        let ssml = build_ssml(request);
        let ssml_message = format!(
            "X-RequestId: {request_id}\r\nContent-Type: application/ssml+xml\r\nX-Timestamp: {timestamp}\r\nPath: ssml\r\n\r\n{ssml}"
        );

        socket
            .send(Message::Text(config.into()))
            .await
            .map_err(|error| AppError::TtsProvider(error.to_string()))?;
        socket
            .send(Message::Text(ssml_message.into()))
            .await
            .map_err(|error| AppError::TtsProvider(error.to_string()))?;

        let mut audio = Vec::new();
        loop {
            let message = tokio::time::timeout(self.timeout, socket.next())
                .await
                .map_err(|_| AppError::TtsProvider("Edge 合成超时".to_string()))?;

            let Some(message) = message else {
                break;
            };
            match message.map_err(|error| AppError::TtsProvider(error.to_string()))? {
                Message::Text(text) => {
                    if text.to_ascii_lowercase().contains("path:turn.end") {
                        break;
                    }
                }
                Message::Binary(data) => {
                    if let Some(header_end) = find_header_end(&data) {
                        let header = String::from_utf8_lossy(&data[..header_end]);
                        if header.to_ascii_lowercase().contains("path:turn.end") {
                            break;
                        }
                        audio.extend_from_slice(&data[header_end + 4..]);
                    } else {
                        audio.extend_from_slice(&data);
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        if audio.is_empty() {
            return Err(AppError::TtsProvider("Edge 返回音频为空".to_string()));
        }

        let _ = socket.close(None).await;
        Ok(SynthesizedAudio {
            bytes: Bytes::from(audio),
            format: "mp3".to_string(),
        })
    }
}

fn build_ssml(request: &SynthesisRequest) -> String {
    let text = escape_xml(&request.text);
    let rate = format!("{:+}%", request.rate_percent);
    let pitch = format!("{:+}Hz", request.pitch_hz);
    let volume = format!("{:+}%", request.volume_percent);
    format!(
        "<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='zh-CN'><voice name='{}'><prosody pitch='{pitch}' rate='{rate}' volume='{volume}'>{text}</prosody></voice></speak>",
        request.provider_voice_id
    )
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn find_header_end(data: &[u8]) -> Option<usize> {
    data.windows(4).position(|window| window == b"\r\n\r\n")
}
