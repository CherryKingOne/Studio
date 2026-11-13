use serde_json::Value;
use sqlx::types::Json;
use sqlx::MySqlPool;

use crate::delivery::exports::model::ExportRow;

pub async fn insert(
    pool: &MySqlPool,
    book_id: u64,
    options: &Value,
    total_chapters: u32,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO exports (book_id, export_type, format, status, options, total_chapters) VALUES (?, 'book_zip', 'zip', 'queued', ?, ?)",
    )
    .bind(book_id)
    .bind(Json(options.clone()))
    .bind(total_chapters)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn find_by_id(pool: &MySqlPool, id: u64) -> Result<Option<ExportRow>, sqlx::Error> {
    sqlx::query_as::<_, ExportRow>("SELECT * FROM exports WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn mark_running(pool: &MySqlPool, id: u64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE exports SET status = 'running' WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_succeeded(
    pool: &MySqlPool,
    id: u64,
    file_id: u64,
    included_chapters: u32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE exports SET status = 'succeeded', file_id = ?, included_chapters = ?, finished_at = UTC_TIMESTAMP(3) WHERE id = ?",
    )
    .bind(file_id)
    .bind(included_chapters)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_failed(
    pool: &MySqlPool,
    id: u64,
    error_message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE exports SET status = 'failed', error_message = ?, finished_at = UTC_TIMESTAMP(3) WHERE id = ?",
    )
    .bind(error_message.chars().take(1000).collect::<String>())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}
