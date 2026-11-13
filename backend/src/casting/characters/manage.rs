use std::collections::HashSet;

use serde_json::{json, Value};

use crate::casting::characters::data;
use crate::casting::characters::detect_rules;
use crate::casting::characters::responses::{
    CharacterDto, CharacterListResponse, CreateCharacterRequest, DetectPayload,
    UpdateCharacterRequest,
};
use crate::casting::characters::types::Character;
use crate::library::books::service as books_service;
use crate::library::chapters::service as chapters_service;
use crate::library::segments::repo as segments_repo;
use crate::library::segments::service as segments_service;
use crate::runtime::error::{AppError, FieldError};
use crate::runtime::state::AppState;
use crate::task_center::queue as jobs_queue;
use crate::task_center::repo as jobs_repo;
use crate::task_center::repo::NewJob;

pub async fn list(state: &AppState, book_id: u64) -> Result<CharacterListResponse, AppError> {
    let rows = data::list_rows(&state.db, book_id).await?;
    let items: Vec<CharacterDto> = rows.into_iter().map(CharacterDto::from).collect();
    Ok(CharacterListResponse {
        total: items.len() as u64,
        items,
    })
}

pub async fn list_models(state: &AppState, book_id: u64) -> Result<Vec<Character>, AppError> {
    Ok(data::list_models(&state.db, book_id).await?)
}

pub async fn create(
    state: &AppState,
    book_id: u64,
    request: CreateCharacterRequest,
) -> Result<CharacterDto, AppError> {
    let _ = books_service::get_model(state, book_id).await?;
    let name = validate_name(&request.name)?;
    if data::find_by_book_name(&state.db, book_id, &name)
        .await?
        .is_some()
    {
        return Err(AppError::Duplicate("同书角色名已存在".to_string()));
    }

    let aliases = normalize_aliases(request.aliases.unwrap_or_default())?;
    let gender = validate_gender(request.gender.as_deref().unwrap_or("unknown"))?;
    let age_group = validate_age(request.age_group.as_deref().unwrap_or("unknown"))?;
    let id = data::insert(
        &state.db,
        book_id,
        &name,
        &aliases,
        &gender,
        &age_group,
        request.description.as_deref(),
        request.default_voice_id,
        request.color.as_deref(),
        "manual",
    )
    .await?;
    get_dto(state, id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    request: UpdateCharacterRequest,
) -> Result<CharacterDto, AppError> {
    let character = get_model(state, id).await?;

    let name = match request.name {
        Some(name) => {
            let name = validate_name(&name)?;
            if name != character.name
                && data::find_by_book_name(&state.db, character.book_id, &name)
                    .await?
                    .is_some()
            {
                return Err(AppError::Duplicate("同书角色名已存在".to_string()));
            }
            name
        }
        None => character.name.clone(),
    };

    let aliases = match request.aliases {
        Some(aliases) => normalize_aliases(aliases)?,
        None => character.aliases.0.clone(),
    };
    let gender = validate_gender(
        request
            .gender
            .as_deref()
            .unwrap_or(character.gender.as_str()),
    )?;
    let age_group = validate_age(
        request
            .age_group
            .as_deref()
            .unwrap_or(character.age_group.as_str()),
    )?;
    let description = request.description.or(character.description.clone());
    let default_voice_id = request.default_voice_id.or(character.default_voice_id);
    let color = request.color.or(character.color.clone());

    if default_voice_id != character.default_voice_id {
        let chapter_ids = segments_service::invalidate_for_character(state, id).await?;
        for chapter_id in chapter_ids {
            chapters_service::clear_audio(state, chapter_id).await?;
        }
    }

    data::update(
        &state.db,
        id,
        &name,
        &aliases,
        &gender,
        &age_group,
        description.as_deref(),
        default_voice_id,
        color.as_deref(),
    )
    .await?;

    get_dto(state, id).await
}

pub async fn delete(state: &AppState, id: u64) -> Result<(), AppError> {
    let _ = get_model(state, id).await?;
    let chapter_ids = segments_service::detach_character(state, id).await?;
    for chapter_id in chapter_ids {
        chapters_service::clear_audio(state, chapter_id).await?;
    }
    data::delete(&state.db, id).await?;
    Ok(())
}

pub async fn create_detect_job(
    state: &AppState,
    book_id: u64,
    overwrite_existing: bool,
    user_id: u64,
) -> Result<u64, AppError> {
    let _ = books_service::get_model(state, book_id).await?;
    if let Some(existing_job_id) =
        jobs_repo::active_job_id(&state.db, book_id, "character_detect").await?
    {
        return Err(AppError::JobAlreadyActive { existing_job_id });
    }

    let payload = DetectPayload {
        book_id,
        overwrite_existing,
    };
    let job = NewJob {
        job_type: "character_detect".to_string(),
        priority: 6,
        book_id: Some(book_id),
        chapter_id: None,
        created_by: Some(user_id),
        payload: serde_json::to_value(&payload)?,
        total_items: 0,
        max_attempts: 3,
    };
    jobs_queue::enqueue(&state.db, &job, &[]).await
}

pub async fn run_detect(
    state: &AppState,
    job: &crate::task_center::model::Job,
) -> Result<Value, String> {
    let payload: DetectPayload =
        serde_json::from_value(job.payload.0.clone()).map_err(|error| error.to_string())?;
    let segments = segments_repo::list_by_book(&state.db, payload.book_id)
        .await
        .map_err(|error| error.to_string())?;
    if segments.is_empty() {
        return Err("书籍没有可识别的段落".to_string());
    }

    let mut characters = data::list_models(&state.db, payload.book_id)
        .await
        .map_err(|error| error.to_string())?;
    let mut existing: HashSet<String> = HashSet::new();
    for character in &characters {
        existing.insert(character.name.clone());
        for alias in &character.aliases.0 {
            existing.insert(alias.clone());
        }
    }

    let candidates = detect_rules::collect_candidates(&segments);
    let mut characters_created = 0u64;
    for (name, count) in candidates {
        if count < 3 || existing.contains(&name) {
            continue;
        }
        data::insert(
            &state.db,
            payload.book_id,
            &name,
            &[],
            "unknown",
            "unknown",
            None,
            None,
            None,
            "rule",
        )
        .await
        .map_err(|error| error.to_string())?;
        existing.insert(name);
        characters_created += 1;
    }

    characters = data::list_models(&state.db, payload.book_id)
        .await
        .map_err(|error| error.to_string())?;
    let total = segments.len();
    let mut assigned = 0u64;
    let mut uncertain = 0u64;
    let mut affected_chapters = HashSet::new();

    for (index, segment) in segments.iter().enumerate() {
        if segment.seg_type != "dialogue" {
            continue;
        }
        if jobs_repo::is_cancel_requested(&state.db, job.id)
            .await
            .map_err(|error| error.to_string())?
        {
            return Err("任务已取消".to_string());
        }
        if segment.speaker_confidence.unwrap_or(0.0) >= 0.999 {
            continue;
        }
        if !payload.overwrite_existing && segment.speaker_id.is_some() {
            continue;
        }

        let guess = detect_rules::guess_speaker(&segments, index, &characters);
        if guess.confidence >= 0.6 && guess.character_id.is_some() {
            segments_service::set_speaker(
                state,
                segment.id,
                guess.character_id,
                Some(guess.confidence),
            )
            .await
            .map_err(|error| error.to_string())?;
            assigned += 1;
            if segment.status == "done" {
                affected_chapters.insert(segment.chapter_id);
            }
        } else {
            if segment.speaker_id.is_none() {
                segments_service::set_speaker(state, segment.id, None, Some(guess.confidence))
                    .await
                    .map_err(|error| error.to_string())?;
            }
            uncertain += 1;
        }

        if index % 20 == 0 {
            let progress = (index * 90 / total.max(1)) as u8;
            let _ = jobs_repo::update_progress(
                &state.db,
                job.id,
                progress.min(90),
                (assigned + uncertain) as u32,
                0,
                &format!("角色识别 {}/{}", index + 1, total),
            )
            .await;
        }
    }

    for chapter_id in affected_chapters {
        chapters_service::clear_audio(state, chapter_id)
            .await
            .map_err(|error| error.to_string())?;
    }

    Ok(json!({
        "book_id": payload.book_id,
        "characters_created": characters_created,
        "segments_assigned": assigned,
        "segments_uncertain": uncertain,
    }))
}

async fn get_model(state: &AppState, id: u64) -> Result<Character, AppError> {
    data::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound { resource: "角色" })
}

async fn get_dto(state: &AppState, id: u64) -> Result<CharacterDto, AppError> {
    let rows = data::list_rows(
        &state.db,
        data::find_by_id(&state.db, id)
            .await?
            .ok_or(AppError::NotFound { resource: "角色" })?
            .book_id,
    )
    .await?;
    rows.into_iter()
        .find(|row| row.character.id == id)
        .map(CharacterDto::from)
        .ok_or(AppError::NotFound { resource: "角色" })
}

fn validate_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 64 {
        return Err(validation("name", "角色名长度必须在 1 到 64 个字符之间"));
    }
    Ok(name.to_string())
}

fn normalize_aliases(aliases: Vec<String>) -> Result<Vec<String>, AppError> {
    let aliases: Vec<String> = aliases
        .into_iter()
        .map(|alias| alias.trim().to_string())
        .filter(|alias| !alias.is_empty())
        .collect();
    if aliases.len() > 20 {
        return Err(validation("aliases", "别名不能超过 20 个"));
    }
    Ok(aliases)
}

fn validate_gender(gender: &str) -> Result<String, AppError> {
    let gender = gender.trim().to_ascii_lowercase();
    if matches!(gender.as_str(), "male" | "female" | "unknown") {
        Ok(gender)
    } else {
        Err(validation("gender", "只能是 male / female / unknown"))
    }
}

fn validate_age(age: &str) -> Result<String, AppError> {
    let age = age.trim().to_ascii_lowercase();
    if matches!(
        age.as_str(),
        "child" | "teen" | "adult" | "elder" | "unknown"
    ) {
        Ok(age)
    } else {
        Err(validation(
            "age_group",
            "只能是 child / teen / adult / elder / unknown",
        ))
    }
}

fn validation(field: &str, message: &str) -> AppError {
    AppError::Validation(vec![FieldError {
        field: field.to_string(),
        message: message.to_string(),
    }])
}
