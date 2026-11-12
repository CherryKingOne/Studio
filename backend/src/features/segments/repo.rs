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
