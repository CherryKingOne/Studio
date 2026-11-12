use std::collections::HashMap;

use bytes::Bytes;
use sha2::{Digest, Sha256};

use crate::common::pagination::Page;
use crate::error::{AppError, FieldError};
use crate::features::files::repo::NewFile;
use crate::features::files::service as files_service;
use crate::features::tts::provider::{SynthesisRequest, VoiceInfo};
use crate::features::voices::dto::{
    CreateVoiceRequest, PreviewRequest, ResolvedVoiceDto, SyncResponse, VoiceDto, VoiceListQuery,
};
use crate::features::voices::model::Voice;
use crate::features::voices::repo;
use crate::state::AppState;

const PREVIEW_TEXT: &str = "你好，这是一段音色试听。";

pub async fn list(state: &AppState, query: VoiceListQuery) -> Result<Page<VoiceDto>, AppError> {
    let (voices, total) = repo::list(&state.db, &query).await?;
    let items = voices.into_iter().map(VoiceDto::from).collect();
    Ok(Page::new(items, query.page(), query.page_size(), total))
}

pub async fn create_manual(
    state: &AppState,
    request: CreateVoiceRequest,
) -> Result<VoiceDto, AppError> {
    let provider = request.provider.trim().to_ascii_lowercase();
    if !matches!(provider.as_str(), "openai" | "minimax" | "stepfun") {
        return Err(validation("provider", "仅支持 openai / minimax / stepfun"));
    }
    if request.provider_voice_id.trim().is_empty() {
        return Err(validation("provider_voice_id", "不能为空"));
    }
    if request.display_name.trim().is_empty() {
        return Err(validation("display_name", "不能为空"));
    }

    let info = VoiceInfo {
        provider_voice_id: request.provider_voice_id.trim().to_string(),
        display_name: request.display_name.trim().to_string(),
        locale: request
            .locale
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "multi".to_string()),
        gender: request
            .gender
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "unknown".to_string()),
        styles: request.styles.unwrap_or_default(),
        extra: None,
    };

    repo::upsert(&state.db, &provider, &info).await?;
    let voice = repo::find_by_provider_voice(&state.db, &provider, &info.provider_voice_id)
        .await?
        .ok_or(AppError::NotFound { resource: "音色" })?;
    Ok(VoiceDto::from(voice))
}

pub async fn sync(state: &AppState, provider: Option<String>) -> Result<SyncResponse, AppError> {
    let provider_name = provider.unwrap_or_else(|| state.tts.provider_name().to_string());
    let engine = state.tts.provider(&provider_name)?;
    let voices = engine.list_voices().await?;
    for voice in &voices {
        repo::upsert(&state.db, &provider_name, voice).await?;
    }
    let message = if voices.is_empty() {
        "该供应商不提供内置音色列表，请通过创建音色接口手动登记".to_string()
    } else {
        format!("已同步 {} 个音色", voices.len())
    };
    Ok(SyncResponse {
        provider: provider_name,
        synced: voices.len() as u64,
        message,
    })
}

pub async fn preview(state: &AppState, request: PreviewRequest) -> Result<Bytes, AppError> {
    let voice = get_voice(state, request.voice_id).await?;
    let text = request
        .text
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| PREVIEW_TEXT.to_string());
    if text.chars().count() > 100 {
        return Err(validation("text", "试听文本不能超过 100 个字符"));
    }

    let engine = state.tts.provider(&voice.provider)?;
    let audio = engine
        .synthesize(&SynthesisRequest {
            text,
            provider_voice_id: voice.provider_voice_id,
            rate_percent: request.rate_percent.unwrap_or(0),
            pitch_hz: request.pitch_hz.unwrap_or(0),
            volume_percent: request.volume_percent.unwrap_or(0),
        })
        .await?;
    Ok(audio.bytes)
}

pub async fn sample(state: &AppState, voice_id: u64) -> Result<Bytes, AppError> {
    let voice = get_voice(state, voice_id).await?;
    if let Some(file_id) = voice.sample_file_id {
        let file = files_service::get(state, file_id).await?;
        return state.storage.get(&file.object_key).await;
    }

    let engine = state.tts.provider(&voice.provider)?;
    let audio = engine
        .synthesize(&SynthesisRequest {
            text: PREVIEW_TEXT.to_string(),
            provider_voice_id: voice.provider_voice_id.clone(),
            rate_percent: 0,
            pitch_hz: 0,
            volume_percent: 0,
        })
        .await?;

    let sha256 = hex::encode(Sha256::digest(&audio.bytes));
    let object_key = format!("samples/{}-{}.mp3", voice.id, &sha256[..8]);
    state
        .storage
        .put(&object_key, audio.bytes.clone(), "audio/mpeg")
        .await?;
    let file_id = files_service::register(
        state,
        NewFile {
            bucket: state.storage.bucket().to_string(),
            object_key,
            file_name: format!("voice-{}.mp3", voice.id),
            mime: "audio/mpeg".to_string(),
            size_bytes: audio.bytes.len() as u64,
            sha256,
            kind: "voice_sample".to_string(),
            duration_ms: None,
        },
    )
    .await?;
    repo::update_sample(&state.db, voice.id, file_id).await?;
    Ok(audio.bytes)
}

pub async fn resolve_for_chapter(
    state: &AppState,
    chapter_id: u64,
    segments: &[crate::features::segments::model::Segment],
) -> Result<HashMap<u64, ResolvedVoiceDto>, AppError> {
    let chapter = crate::features::chapters::service::get_model(state, chapter_id).await?;
    let book = crate::features::books::service::get_model(state, chapter.book_id).await?;
    let characters = crate::features::characters::service::list_models(state, chapter.book_id)
        .await
        .unwrap_or_default();
    let voices = repo::list_all_enabled(&state.db).await?;

    let voice_by_id: HashMap<u64, Voice> =
        voices.into_iter().map(|voice| (voice.id, voice)).collect();
    let character_by_id: HashMap<u64, _> = characters
        .into_iter()
        .map(|character| (character.id, character))
        .collect();

    let system_voice_id = state.config.tts.default_narrator_voice_id.clone();
    let system_provider = state.config.tts.default_provider.clone();

    let mut resolved = HashMap::new();
    for segment in segments {
        let mut source = "fallback";
        let mut voice_id = segment.voice_id;

        if voice_id.is_none() && segment.seg_type == "dialogue" {
            if let Some(speaker_id) = segment.speaker_id {
                voice_id = character_by_id
                    .get(&speaker_id)
                    .and_then(|character| character.default_voice_id);
                if voice_id.is_some() {
                    source = "character_default";
                }
            }
        }

        if voice_id.is_none() {
            voice_id = if segment.seg_type == "narration" {
                chapter.narrator_voice_id.or(book.default_narrator_voice_id)
            } else {
                book.default_narrator_voice_id
            };
            if voice_id.is_some() {
                source = "book_narrator";
            }
        }

        let dto = if let Some(id) = voice_id {
            match voice_by_id.get(&id) {
                Some(voice) => ResolvedVoiceDto {
                    voice_id: Some(voice.id),
                    provider: voice.provider.clone(),
                    provider_voice_id: Some(voice.provider_voice_id.clone()),
                    display_name: Some(voice.display_name.clone()),
                    source: source.to_string(),
                },
                None => ResolvedVoiceDto {
                    voice_id: Some(id),
                    provider: system_provider.clone(),
                    provider_voice_id: None,
                    display_name: None,
                    source: "missing_voice".to_string(),
                },
            }
        } else {
            ResolvedVoiceDto {
                voice_id: None,
                provider: system_provider.clone(),
                provider_voice_id: Some(system_voice_id.clone()),
                display_name: None,
                source: "system_default".to_string(),
            }
        };
        resolved.insert(segment.id, dto);
    }
    Ok(resolved)
}

pub async fn get_voice(state: &AppState, id: u64) -> Result<Voice, AppError> {
    repo::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound { resource: "音色" })
}

fn validation(field: &str, message: &str) -> AppError {
    AppError::Validation(vec![FieldError {
        field: field.to_string(),
        message: message.to_string(),
    }])
}
