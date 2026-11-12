use serde_json::{json, Value};
use sqlx::types::Json;
use sqlx::{MySql, MySqlPool, QueryBuilder};

use crate::features::tts::provider::VoiceInfo;
use crate::features::voices::dto::VoiceListQuery;
use crate::features::voices::model::Voice;

pub async fn list(
    pool: &MySqlPool,
    query: &VoiceListQuery,
) -> Result<(Vec<Voice>, u64), sqlx::Error> {
    let mut qb = QueryBuilder::<MySql>::new("SELECT * FROM voices WHERE 1 = 1");
    let mut count_qb = QueryBuilder::<MySql>::new("SELECT COUNT(*) FROM voices WHERE 1 = 1");

    if let Some(provider) = query
        .provider
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        qb.push(" AND provider = ").push_bind(provider);
        count_qb.push(" AND provider = ").push_bind(provider);
    }
    if let Some(locale) = query
        .locale
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        qb.push(" AND locale = ").push_bind(locale);
        count_qb.push(" AND locale = ").push_bind(locale);
    }
    if let Some(gender) = query
        .gender
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        qb.push(" AND gender = ").push_bind(gender);
        count_qb.push(" AND gender = ").push_bind(gender);
    }
    if let Some(keyword) = query
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let pattern = format!("%{keyword}%");
        qb.push(" AND (display_name LIKE ")
            .push_bind(pattern.clone())
            .push(" OR provider_voice_id LIKE ")
            .push_bind(pattern.clone())
            .push(")");
        count_qb
            .push(" AND (display_name LIKE ")
            .push_bind(pattern.clone())
            .push(" OR provider_voice_id LIKE ")
            .push_bind(pattern)
            .push(")");
    }
    if let Some(enabled) = query.enabled {
        qb.push(" AND enabled = ").push_bind(enabled);
        count_qb.push(" AND enabled = ").push_bind(enabled);
    }

    qb.push(" ORDER BY locale ASC, display_name ASC LIMIT ")
        .push_bind(query.page_size() as i64)
        .push(" OFFSET ")
        .push_bind(query.offset() as i64);

    let voices = qb.build_query_as::<Voice>().fetch_all(pool).await?;
    let total = count_qb.build_query_scalar::<u64>().fetch_one(pool).await?;
    Ok((voices, total))
}

pub async fn find_by_id(pool: &MySqlPool, id: u64) -> Result<Option<Voice>, sqlx::Error> {
    sqlx::query_as::<_, Voice>("SELECT * FROM voices WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_provider_voice(
    pool: &MySqlPool,
    provider: &str,
    provider_voice_id: &str,
) -> Result<Option<Voice>, sqlx::Error> {
    sqlx::query_as::<_, Voice>(
        "SELECT * FROM voices WHERE provider = ? AND provider_voice_id = ?",
    )
    .bind(provider)
    .bind(provider_voice_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_all_enabled(pool: &MySqlPool) -> Result<Vec<Voice>, sqlx::Error> {
    sqlx::query_as::<_, Voice>("SELECT * FROM voices WHERE enabled = 1")
        .fetch_all(pool)
        .await
}

pub async fn upsert(
    pool: &MySqlPool,
    provider: &str,
    voice: &VoiceInfo,
) -> Result<(), sqlx::Error> {
    let styles = Json(json!(voice.styles));
    let extra = voice.extra.clone().map(Json);
    sqlx::query(
        r#"
        INSERT INTO voices (provider, provider_voice_id, display_name, locale, gender, styles, extra, enabled)
        VALUES (?, ?, ?, ?, ?, ?, ?, 1)
        ON DUPLICATE KEY UPDATE
            display_name = VALUES(display_name),
            locale = VALUES(locale),
            gender = VALUES(gender),
            styles = VALUES(styles),
            extra = VALUES(extra),
            enabled = 1
        "#,
    )
    .bind(provider)
    .bind(&voice.provider_voice_id)
    .bind(&voice.display_name)
    .bind(&voice.locale)
    .bind(&voice.gender)
    .bind(styles)
    .bind(extra)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_sample(
    pool: &MySqlPool,
    id: u64,
    file_id: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE voices SET sample_file_id = ? WHERE id = ?")
        .bind(file_id)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub fn styles_json(styles: &[String]) -> Value {
    json!(styles)
}
