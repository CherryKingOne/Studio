use sqlx::{MySql, MySqlPool, QueryBuilder, Transaction};

use crate::common::text::TextSegment;
use crate::features::segments::dto::SegmentListQuery;
use crate::features::segments::model::Segment;

pub struct SegmentPatch {
    pub text: String,
    pub char_count: u32,
    pub seg_type: String,
    pub speaker_id: Option<u64>,
    pub speaker_confidence: Option<f32>,
    pub voice_id: Option<u64>,
    pub emotion: String,
    pub rate_percent: i16,
    pub pitch_hz: i16,
    pub volume_percent: i16,
}

pub async fn list_by_chapter(
    pool: &MySqlPool,
    chapter_id: u64,
    query: &SegmentListQuery,
) -> Result<Vec<Segment>, sqlx::Error> {
    let mut qb = QueryBuilder::<MySql>::new("SELECT * FROM segments WHERE chapter_id = ");
    qb.push_bind(chapter_id);

    if let Some(seg_type) = query
        .seg_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        qb.push(" AND seg_type = ").push_bind(seg_type);
    }
    if let Some(status) = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        qb.push(" AND status = ").push_bind(status);
    }
    if let Some(speaker_id) = query.speaker_id {
        qb.push(" AND speaker_id = ").push_bind(speaker_id);
    }
    qb.push(" ORDER BY idx ASC LIMIT 2000");

    qb.build_query_as::<Segment>().fetch_all(pool).await
}

pub async fn find_by_id(pool: &MySqlPool, id: u64) -> Result<Option<Segment>, sqlx::Error> {
    sqlx::query_as::<_, Segment>("SELECT * FROM segments WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert_batch_tx(
    tx: &mut Transaction<'_, MySql>,
    chapter_id: u64,
    segments: &[TextSegment],
) -> Result<u64, sqlx::Error> {
    for (position, segment) in segments.iter().enumerate() {
        let idx = position as u32 + 1;
        let char_count = segment.text.chars().count() as u32;
        sqlx::query(
            "INSERT INTO segments (chapter_id, idx, text, char_count, seg_type, status) VALUES (?, ?, ?, ?, ?, 'pending')",
        )
        .bind(chapter_id)
        .bind(idx)
        .bind(&segment.text)
        .bind(char_count)
        .bind(segment.seg_type)
        .execute(&mut **tx)
        .await?;
    }
    Ok(segments.len() as u64)
}

pub async fn delete_by_chapter_tx(
    tx: &mut Transaction<'_, MySql>,
    chapter_id: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM segments WHERE chapter_id = ?")
        .bind(chapter_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn update_fields_tx(
    tx: &mut Transaction<'_, MySql>,
    id: u64,
    patch: &SegmentPatch,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE segments
           SET text = ?, char_count = ?, seg_type = ?, speaker_id = ?,
               speaker_confidence = ?, voice_id = ?, emotion = ?,
               rate_percent = ?, pitch_hz = ?, volume_percent = ?,
               status = 'pending', audio_file_id = NULL, audio_duration_ms = NULL,
               cache_key = NULL, attempts = 0, error_message = NULL
         WHERE id = ?
        "#,
    )
    .bind(&patch.text)
    .bind(patch.char_count)
    .bind(&patch.seg_type)
    .bind(patch.speaker_id)
    .bind(patch.speaker_confidence)
    .bind(patch.voice_id)
    .bind(&patch.emotion)
    .bind(patch.rate_percent)
    .bind(patch.pitch_hz)
    .bind(patch.volume_percent)
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn has_done_by_chapter(pool: &MySqlPool, chapter_id: u64) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, u64>(
        "SELECT COUNT(*) FROM segments WHERE chapter_id = ? AND status = 'done'",
    )
    .bind(chapter_id)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

pub async fn count_by_chapter(pool: &MySqlPool, chapter_id: u64) -> Result<u64, sqlx::Error> {
    let count = sqlx::query_scalar::<_, u64>("SELECT COUNT(*) FROM segments WHERE chapter_id = ?")
        .bind(chapter_id)
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn list_by_book(pool: &MySqlPool, book_id: u64) -> Result<Vec<Segment>, sqlx::Error> {
    sqlx::query_as::<_, Segment>(
        r#"
        SELECT s.* FROM segments s
          JOIN chapters c ON c.id = s.chapter_id
         WHERE c.book_id = ?
         ORDER BY c.idx ASC, s.idx ASC
        "#,
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
}

pub async fn find_by_ids(pool: &MySqlPool, ids: &[u64]) -> Result<Vec<Segment>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut qb = QueryBuilder::<MySql>::new("SELECT * FROM segments WHERE id IN (");
    {
        let mut separated = qb.separated(", ");
        for id in ids {
            separated.push_bind(*id);
        }
    }
    qb.push(") ORDER BY chapter_id ASC, idx ASC");
    qb.build_query_as::<Segment>().fetch_all(pool).await
}

pub async fn claim_for_synthesis(pool: &MySqlPool, id: u64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE segments SET status = 'synthesizing', updated_at = UTC_TIMESTAMP(3) WHERE id = ? AND status IN ('pending', 'failed', 'done')",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn mark_done_tx(
    tx: &mut Transaction<'_, MySql>,
    id: u64,
    file_id: u64,
    duration_ms: u32,
    cache_key: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE segments
           SET status = 'done', audio_file_id = ?, audio_duration_ms = ?, cache_key = ?,
               error_message = NULL, updated_at = UTC_TIMESTAMP(3)
         WHERE id = ?
        "#,
    )
    .bind(file_id)
    .bind(duration_ms)
    .bind(cache_key)
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn mark_failed(
    pool: &MySqlPool,
    id: u64,
    attempts: u8,
    error_message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE segments SET status = 'failed', attempts = ?, error_message = ?, updated_at = UTC_TIMESTAMP(3) WHERE id = ?",
    )
    .bind(attempts)
    .bind(error_message.chars().take(500).collect::<String>())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn chapter_counts(
    pool: &MySqlPool,
    chapter_id: u64,
) -> Result<(u64, u64, u64), sqlx::Error> {
    let total = sqlx::query_scalar::<_, u64>("SELECT COUNT(*) FROM segments WHERE chapter_id = ?")
        .bind(chapter_id)
        .fetch_one(pool)
        .await?;
    let done = sqlx::query_scalar::<_, u64>(
        "SELECT COUNT(*) FROM segments WHERE chapter_id = ? AND status = 'done'",
    )
    .bind(chapter_id)
    .fetch_one(pool)
    .await?;
    let failed = sqlx::query_scalar::<_, u64>(
        "SELECT COUNT(*) FROM segments WHERE chapter_id = ? AND status = 'failed'",
    )
    .bind(chapter_id)
    .fetch_one(pool)
    .await?;
    Ok((total, done, failed))
}

pub async fn set_speaker(
    pool: &MySqlPool,
    id: u64,
    speaker_id: Option<u64>,
    confidence: Option<f32>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE segments
           SET audio_file_id = IF(status = 'done', NULL, audio_file_id),
               audio_duration_ms = IF(status = 'done', NULL, audio_duration_ms),
               cache_key = IF(status = 'done', NULL, cache_key),
               status = IF(status = 'done', 'pending', status),
               speaker_id = ?, speaker_confidence = ?
         WHERE id = ?
        "#,
    )
    .bind(speaker_id)
    .bind(confidence)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn invalidate_for_character_tx(
    tx: &mut Transaction<'_, MySql>,
    character_id: u64,
    only_without_override: bool,
) -> Result<Vec<u64>, sqlx::Error> {
    let mut qb =
        QueryBuilder::<MySql>::new("SELECT DISTINCT chapter_id FROM segments WHERE speaker_id = ");
    qb.push_bind(character_id).push(" AND status = 'done'");
    if only_without_override {
        qb.push(" AND voice_id IS NULL");
    }
    let chapter_ids: Vec<u64> = qb.build_query_scalar().fetch_all(&mut **tx).await?;

    let mut update =
        QueryBuilder::<MySql>::new("UPDATE segments SET status = 'pending', audio_file_id = NULL, audio_duration_ms = NULL, cache_key = NULL, attempts = 0, error_message = NULL WHERE speaker_id = ");
    update.push_bind(character_id);
    if only_without_override {
        update.push(" AND voice_id IS NULL");
    }
    update.build().execute(&mut **tx).await?;
    Ok(chapter_ids)
}

pub async fn detach_character_tx(
    tx: &mut Transaction<'_, MySql>,
    character_id: u64,
) -> Result<Vec<u64>, sqlx::Error> {
    let chapter_ids: Vec<u64> =
        sqlx::query_scalar("SELECT DISTINCT chapter_id FROM segments WHERE speaker_id = ?")
            .bind(character_id)
            .fetch_all(&mut **tx)
            .await?;

    sqlx::query(
        r#"
        UPDATE segments
           SET audio_file_id = IF(status = 'done', NULL, audio_file_id),
               audio_duration_ms = IF(status = 'done', NULL, audio_duration_ms),
               cache_key = IF(status = 'done', NULL, cache_key),
               status = IF(status = 'done', 'pending', status),
               speaker_id = NULL, speaker_confidence = NULL
         WHERE speaker_id = ?
        "#,
    )
    .bind(character_id)
    .execute(&mut **tx)
    .await?;
    Ok(chapter_ids)
}
