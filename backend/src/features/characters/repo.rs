use serde_json::json;
use sqlx::types::Json;
use sqlx::MySqlPool;

use crate::features::characters::model::{Character, CharacterRow};

pub async fn list_rows(pool: &MySqlPool, book_id: u64) -> Result<Vec<CharacterRow>, sqlx::Error> {
    sqlx::query_as::<_, CharacterRow>(
        r#"
        SELECT c.*, (SELECT COUNT(*) FROM segments s WHERE s.speaker_id = c.id) AS segment_count
          FROM characters c
         WHERE c.book_id = ?
         ORDER BY c.created_at ASC
        "#,
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
}

pub async fn list_models(pool: &MySqlPool, book_id: u64) -> Result<Vec<Character>, sqlx::Error> {
    sqlx::query_as::<_, Character>(
        "SELECT * FROM characters WHERE book_id = ? ORDER BY created_at ASC",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
}

pub async fn find_by_id(pool: &MySqlPool, id: u64) -> Result<Option<Character>, sqlx::Error> {
    sqlx::query_as::<_, Character>("SELECT * FROM characters WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_book_name(
    pool: &MySqlPool,
    book_id: u64,
    name: &str,
) -> Result<Option<Character>, sqlx::Error> {
    sqlx::query_as::<_, Character>("SELECT * FROM characters WHERE book_id = ? AND name = ?")
        .bind(book_id)
        .bind(name)
        .fetch_optional(pool)
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn insert(
    pool: &MySqlPool,
    book_id: u64,
    name: &str,
    aliases: &[String],
    gender: &str,
    age_group: &str,
    description: Option<&str>,
    default_voice_id: Option<u64>,
    color: Option<&str>,
    source: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO characters (book_id, name, aliases, gender, age_group, description, default_voice_id, color, source)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(book_id)
    .bind(name)
    .bind(Json(json!(aliases)))
    .bind(gender)
    .bind(age_group)
    .bind(description)
    .bind(default_voice_id)
    .bind(color)
    .bind(source)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

#[allow(clippy::too_many_arguments)]
pub async fn update(
    pool: &MySqlPool,
    id: u64,
    name: &str,
    aliases: &[String],
    gender: &str,
    age_group: &str,
    description: Option<&str>,
    default_voice_id: Option<u64>,
    color: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE characters
           SET name = ?, aliases = ?, gender = ?, age_group = ?, description = ?,
               default_voice_id = ?, color = ?, updated_at = UTC_TIMESTAMP(3)
         WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(Json(json!(aliases)))
    .bind(gender)
    .bind(age_group)
    .bind(description)
    .bind(default_voice_id)
    .bind(color)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &MySqlPool, id: u64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM characters WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
