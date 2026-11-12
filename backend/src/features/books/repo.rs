use serde_json::Value;
use sqlx::types::Json;
use sqlx::{MySql, MySqlPool, QueryBuilder};

use crate::features::books::dto::BookListQuery;
use crate::features::books::model::Book;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BookStatsRow {
    pub total_chapters: u64,
    pub segmented_chapters: u64,
    pub tts_done_chapters: u64,
    pub merged_chapters: u64,
    pub total_segments: u64,
    pub done_segments: u64,
    pub failed_segments: u64,
    pub active_job_count: u64,
}

pub async fn list(
    pool: &MySqlPool,
    query: &BookListQuery,
) -> Result<(Vec<Book>, u64), sqlx::Error> {
    let mut qb = QueryBuilder::<MySql>::new("SELECT * FROM books WHERE deleted_at IS NULL");
    let mut count_qb =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) FROM books WHERE deleted_at IS NULL");

    if let Some(keyword) = query
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let pattern = format!("%{keyword}%");
        qb.push(" AND (title LIKE ")
            .push_bind(pattern.clone())
            .push(" OR author LIKE ")
            .push_bind(pattern.clone())
            .push(")");
        count_qb
            .push(" AND (title LIKE ")
            .push_bind(pattern.clone())
            .push(" OR author LIKE ")
            .push_bind(pattern)
            .push(")");
    }

    if let Some(status) = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        qb.push(" AND status = ").push_bind(status);
        count_qb.push(" AND status = ").push_bind(status);
    }

    if !query.include_archived.unwrap_or(false) {
        qb.push(" AND status <> 'archived'");
        count_qb.push(" AND status <> 'archived'");
    }

    qb.push(" ORDER BY created_at DESC LIMIT ")
        .push_bind(query.page_size() as i64)
        .push(" OFFSET ")
        .push_bind(query.offset() as i64);

    let books = qb.build_query_as::<Book>().fetch_all(pool).await?;
    let total = count_qb.build_query_scalar::<u64>().fetch_one(pool).await?;

    Ok((books, total))
}

pub async fn find_by_id(pool: &MySqlPool, id: u64) -> Result<Option<Book>, sqlx::Error> {
    sqlx::query_as::<_, Book>("SELECT * FROM books WHERE id = ? AND deleted_at IS NULL")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn exists_title(pool: &MySqlPool, title: &str) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, u64>(
        "SELECT COUNT(*) FROM books WHERE title = ? AND deleted_at IS NULL",
    )
    .bind(title)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

pub async fn insert(
    pool: &MySqlPool,
    title: &str,
    author: Option<&str>,
    description: Option<&str>,
    settings: &Value,
    created_by: u64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO books (title, author, description, language, settings, status, created_by)
        VALUES (?, ?, ?, 'zh-CN', ?, 'draft', ?)
        "#,
    )
    .bind(title)
    .bind(author)
    .bind(description)
    .bind(Json(settings.clone()))
    .bind(created_by)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

#[allow(clippy::too_many_arguments)]
pub async fn update(
    pool: &MySqlPool,
    id: u64,
    title: &str,
    author: Option<&str>,
    description: Option<&str>,
    default_narrator_voice_id: Option<u64>,
    settings: &Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE books
           SET title = ?, author = ?, description = ?,
               default_narrator_voice_id = ?, settings = ?,
               updated_at = UTC_TIMESTAMP(3)
         WHERE id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(title)
    .bind(author)
    .bind(description)
    .bind(default_narrator_voice_id)
    .bind(Json(settings.clone()))
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_cover(pool: &MySqlPool, id: u64, file_id: u64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE books SET cover_file_id = ? WHERE id = ? AND deleted_at IS NULL")
        .bind(file_id)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_import_source_tx(
    tx: &mut sqlx::Transaction<'_, MySql>,
    id: u64,
    file_id: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE books SET source_file_id = ?, status = 'importing', error_message = NULL WHERE id = ?",
    )
    .bind(file_id)
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn update_status(
    pool: &MySqlPool,
    id: u64,
    status: &str,
    error_message: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE books SET status = ?, error_message = ? WHERE id = ?")
        .bind(status)
        .bind(error_message)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_stats(
    pool: &MySqlPool,
    id: u64,
    total_chapters: u32,
    total_segments: u32,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE books SET total_chapters = ?, total_segments = ?, status = ?, error_message = NULL WHERE id = ?",
    )
    .bind(total_chapters)
    .bind(total_segments)
    .bind(status)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn soft_delete(pool: &MySqlPool, id: u64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE books SET deleted_at = UTC_TIMESTAMP(3) WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn count_active_jobs(pool: &MySqlPool, book_id: u64) -> Result<u64, sqlx::Error> {
    let count = sqlx::query_scalar::<_, u64>(
        "SELECT COUNT(*) FROM jobs WHERE book_id = ? AND status IN ('queued', 'running')",
    )
    .bind(book_id)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn stats(pool: &MySqlPool, book_id: u64) -> Result<BookStatsRow, sqlx::Error> {
    sqlx::query_as::<_, BookStatsRow>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM chapters WHERE book_id = ?) AS total_chapters,
            (SELECT COUNT(*) FROM chapters
              WHERE book_id = ?
                AND status IN ('segmented', 'tts_partial', 'tts_done', 'merged')) AS segmented_chapters,
            (SELECT COUNT(*) FROM chapters
              WHERE book_id = ? AND status IN ('tts_done', 'merged')) AS tts_done_chapters,
            (SELECT COUNT(*) FROM chapters WHERE book_id = ? AND status = 'merged') AS merged_chapters,
            (SELECT COUNT(*) FROM segments s
               JOIN chapters c ON c.id = s.chapter_id
              WHERE c.book_id = ?) AS total_segments,
            (SELECT COUNT(*) FROM segments s
               JOIN chapters c ON c.id = s.chapter_id
              WHERE c.book_id = ? AND s.status = 'done') AS done_segments,
            (SELECT COUNT(*) FROM segments s
               JOIN chapters c ON c.id = s.chapter_id
              WHERE c.book_id = ? AND s.status = 'failed') AS failed_segments,
            (SELECT COUNT(*) FROM jobs
              WHERE book_id = ? AND status IN ('queued', 'running')) AS active_job_count
        "#,
    )
    .bind(book_id)
    .bind(book_id)
    .bind(book_id)
    .bind(book_id)
    .bind(book_id)
    .bind(book_id)
    .bind(book_id)
    .bind(book_id)
    .fetch_one(pool)
    .await
}
