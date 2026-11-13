use chrono::NaiveDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Chapter {
    pub id: u64,
    pub book_id: u64,
    pub idx: u32,
    pub title: String,
    pub raw_text: String,
    pub char_count: u32,
    pub status: String,
    pub narrator_voice_id: Option<u64>,
    pub audio_file_id: Option<u64>,
    pub audio_duration_ms: Option<u32>,
    pub audio_rev: u32,
    pub tts_done_segments: u32,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChapterRow {
    #[sqlx(flatten)]
    pub chapter: Chapter,
    pub segment_count: u64,
}
use sqlx::{MySql, MySqlPool, QueryBuilder, Transaction};

use crate::library::chapters::chapter_types::ChapterListQuery;

pub async fn list(
    pool: &MySqlPool,
    book_id: u64,
    query: &ChapterListQuery,
) -> Result<(Vec<ChapterRow>, u64), sqlx::Error> {
    let mut qb = QueryBuilder::<MySql>::new(
        "SELECT c.*, (SELECT CAST(COUNT(*) AS UNSIGNED) FROM segments s WHERE s.chapter_id = c.id) AS segment_count FROM chapters c JOIN books b ON b.id = c.book_id WHERE c.book_id = ",
    );
    qb.push_bind(book_id);
    qb.push(" AND b.deleted_at IS NULL");
    let mut count_qb = QueryBuilder::<MySql>::new(
        "SELECT CAST(COUNT(*) AS UNSIGNED) FROM chapters c JOIN books b ON b.id = c.book_id WHERE c.book_id = ",
    );
    count_qb.push_bind(book_id);
    count_qb.push(" AND b.deleted_at IS NULL");

    if let Some(status) = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        qb.push(" AND c.status = ").push_bind(status);
        count_qb.push(" AND c.status = ").push_bind(status);
    }
    if let Some(keyword) = query
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let pattern = format!("%{keyword}%");
        qb.push(" AND c.title LIKE ").push_bind(pattern.clone());
        count_qb.push(" AND c.title LIKE ").push_bind(pattern);
    }

    qb.push(" ORDER BY c.idx ASC LIMIT ")
        .push_bind(query.page_size() as i64)
        .push(" OFFSET ")
        .push_bind(query.offset() as i64);

    let rows = qb.build_query_as::<ChapterRow>().fetch_all(pool).await?;
    let total = count_qb.build_query_scalar::<u64>().fetch_one(pool).await?;
    Ok((rows, total))
}

pub async fn find_by_id(pool: &MySqlPool, id: u64) -> Result<Option<Chapter>, sqlx::Error> {
    sqlx::query_as::<_, Chapter>(
        "SELECT c.* FROM chapters c JOIN books b ON b.id = c.book_id WHERE c.id = ? AND b.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn insert_tx(
    tx: &mut Transaction<'_, MySql>,
    book_id: u64,
    idx: u32,
    title: &str,
    raw_text: &str,
    char_count: u32,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO chapters (book_id, idx, title, raw_text, char_count, status) VALUES (?, ?, ?, ?, ?, 'pending')",
    )
    .bind(book_id)
    .bind(idx)
    .bind(title)
    .bind(raw_text)
    .bind(char_count)
    .execute(&mut **tx)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn update_content(
    pool: &MySqlPool,
    id: u64,
    title: &str,
    raw_text: &str,
    char_count: u32,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE chapters SET title = ?, raw_text = ?, char_count = ?, status = ? WHERE id = ?",
    )
    .bind(title)
    .bind(raw_text)
    .bind(char_count)
    .bind(status)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn clear_audio_tx(
    tx: &mut Transaction<'_, MySql>,
    chapter_id: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE chapters SET audio_file_id = NULL, audio_duration_ms = NULL, audio_rev = audio_rev + 1 WHERE id = ?",
    )
    .bind(chapter_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn update_after_resegment_tx(
    tx: &mut Transaction<'_, MySql>,
    chapter_id: u64,
    char_count: u32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE chapters
           SET status = 'segmented', char_count = ?, tts_done_segments = 0,
               audio_file_id = NULL, audio_duration_ms = NULL,
               audio_rev = audio_rev + 1, error_message = NULL
         WHERE id = ?
        "#,
    )
    .bind(char_count)
    .bind(chapter_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn count_by_book(pool: &MySqlPool, book_id: u64) -> Result<u64, sqlx::Error> {
    let count = sqlx::query_scalar::<_, u64>(
        "SELECT CAST(COUNT(*) AS UNSIGNED) FROM chapters WHERE book_id = ?",
    )
    .bind(book_id)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn delete_by_book(pool: &MySqlPool, book_id: u64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM chapters WHERE book_id = ?")
        .bind(book_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_status(pool: &MySqlPool, id: u64, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE chapters SET status = ? WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn invalidate_after_tts(pool: &MySqlPool, id: u64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE chapters SET audio_file_id = NULL, audio_duration_ms = NULL, audio_rev = audio_rev + 1 WHERE id = ?",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_merged_by_book(
    pool: &MySqlPool,
    book_id: u64,
) -> Result<Vec<Chapter>, sqlx::Error> {
    sqlx::query_as::<_, Chapter>(
        "SELECT * FROM chapters WHERE book_id = ? AND status = 'merged' AND audio_file_id IS NOT NULL ORDER BY idx ASC",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
}

pub async fn update_audio(
    pool: &MySqlPool,
    id: u64,
    file_id: u64,
    duration_ms: u32,
    audio_rev: u32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE chapters SET audio_file_id = ?, audio_duration_ms = ?, audio_rev = ?, status = 'merged', error_message = NULL WHERE id = ?",
    )
    .bind(file_id)
    .bind(duration_ms)
    .bind(audio_rev)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}
