use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use serde::Serialize;
use serde_json::Value;

use crate::config::TtsSection;
use crate::error::AppError;

pub mod openai;

#[derive(Debug, Clone, Serialize)]
pub struct VoiceInfo {
    pub provider_voice_id: String,
    pub display_name: String,
    pub locale: String,
    pub gender: String,
    pub styles: Vec<String>,
    pub extra: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct SynthesisRequest {
    pub text: String,
    pub provider_voice_id: String,
    pub rate_percent: i16,
    pub pitch_hz: i16,
    pub volume_percent: i16,
}

#[derive(Debug, Clone)]
pub struct SynthesizedAudio {
    pub bytes: Bytes,
    pub format: String,
}

#[async_trait]
pub trait TtsProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    async fn list_voices(&self) -> Result<Vec<VoiceInfo>, AppError>;
    async fn synthesize(&self, request: &SynthesisRequest) -> Result<SynthesizedAudio, AppError>;
}

pub struct TtsRegistry {
    providers: HashMap<String, Arc<dyn TtsProvider>>,
    default_provider: String,
}

impl TtsRegistry {
    pub fn new(config: &TtsSection) -> Self {
        let mut providers: HashMap<String, Arc<dyn TtsProvider>> = HashMap::new();
        providers.insert(
            "openai".to_string(),
            Arc::new(openai::OpenAiTtsProvider::new(config)),
        );
        let default_provider = if providers.contains_key(&config.default_provider) {
            config.default_provider.clone()
        } else {
            "openai".to_string()
        };
        Self {
            providers,
            default_provider,
        }
    }

    pub fn provider(&self, name: &str) -> Result<Arc<dyn TtsProvider>, AppError> {
        self.providers.get(name).cloned().ok_or_else(|| {
            AppError::Validation(vec![crate::error::FieldError {
                field: "provider".to_string(),
                message: format!("不支持的 TTS 引擎: {name}"),
            }])
        })
    }

    pub fn provider_name(&self) -> &str {
        &self.default_provider
    }
}
