use chrono::NaiveDateTime;
use sqlx::{MySql, MySqlPool, QueryBuilder, Transaction};

use crate::library::books::book_data::Book;
use crate::recycle_bin::dto::RecycleBinListQuery;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DeletedBookRow {
    #[sqlx(flatten)]
    pub book: Book,
    pub deleted_by_username: Option<String>,
    pub deleted_by_display_name: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SummaryRow {
    pub total: u64,
    pub oldest_deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FileObjectRow {
    pub id: u64,
    pub object_key: String,
}

pub async fn list(
    pool: &MySqlPool,
    query: &RecycleBinListQuery,
) -> Result<(Vec<DeletedBookRow>, u64), sqlx::Error> {
    let mut qb = QueryBuilder::<MySql>::new(
        r#"
        SELECT b.*, u.username AS deleted_by_username, u.display_name AS deleted_by_display_name
          FROM books b
          LEFT JOIN users u ON u.id = b.deleted_by
         WHERE b.deleted_at IS NOT NULL
        "#,
    );
    let mut count_qb = QueryBuilder::<MySql>::new(
        "SELECT CAST(COUNT(*) AS UNSIGNED) FROM books b WHERE b.deleted_at IS NOT NULL",
    );

    if let Some(keyword) = query
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let pattern = format!("%{keyword}%");
        qb.push(" AND (b.title LIKE ")
            .push_bind(pattern.clone())
            .push(" OR b.author LIKE ")
            .push_bind(pattern.clone())
            .push(")");
        count_qb
            .push(" AND (b.title LIKE ")
            .push_bind(pattern.clone())
            .push(" OR b.author LIKE ")
            .push_bind(pattern)
            .push(")");
    }

    qb.push(" ORDER BY b.deleted_at DESC, b.id DESC LIMIT ")
        .push_bind(query.page_size() as i64)
        .push(" OFFSET ")
        .push_bind(query.offset() as i64);

    let rows = qb
        .build_query_as::<DeletedBookRow>()
        .fetch_all(pool)
        .await?;
    let total = count_qb.build_query_scalar::<u64>().fetch_one(pool).await?;
    Ok((rows, total))
}

pub async fn summary(pool: &MySqlPool) -> Result<SummaryRow, sqlx::Error> {
    sqlx::query_as::<_, SummaryRow>(
        "SELECT CAST(COUNT(*) AS UNSIGNED) AS total, MIN(deleted_at) AS oldest_deleted_at FROM books WHERE deleted_at IS NOT NULL",
    )
    .fetch_one(pool)
    .await
}

pub async fn find_in_trash(pool: &MySqlPool, id: u64) -> Result<Option<Book>, sqlx::Error> {
    sqlx::query_as::<_, Book>("SELECT * FROM books WHERE id = ? AND deleted_at IS NOT NULL")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_any(pool: &MySqlPool, id: u64) -> Result<Option<Book>, sqlx::Error> {
    sqlx::query_as::<_, Book>("SELECT * FROM books WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_in_trash_for_update(
    tx: &mut Transaction<'_, MySql>,
    id: u64,
) -> Result<Option<Book>, sqlx::Error> {
    sqlx::query_as::<_, Book>(
        "SELECT * FROM books WHERE id = ? AND deleted_at IS NOT NULL FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn find_any_tx(
    tx: &mut Transaction<'_, MySql>,
    id: u64,
) -> Result<Option<Book>, sqlx::Error> {
    sqlx::query_as::<_, Book>("SELECT * FROM books WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
}

pub async fn title_taken_tx(
    tx: &mut Transaction<'_, MySql>,
    title: &str,
    exclude_id: u64,
) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, u64>(
        "SELECT CAST(COUNT(*) AS UNSIGNED) FROM books WHERE title = ? AND deleted_at IS NULL AND id <> ?",
    )
    .bind(title)
    .bind(exclude_id)
    .fetch_one(&mut **tx)
    .await?;
    Ok(count > 0)
}

/// 在保留 255 字符上限的前提下追加“（恢复）”后缀。
pub fn append_restore_suffix(title: &str, suffix: &str) -> String {
    let limit = 255usize.saturating_sub(suffix.chars().count());
    let base: String = title.chars().take(limit).collect();
    let base = base.trim_end();
    let base = if base.is_empty() { "未命名" } else { base };
    format!("{base}{suffix}")
}

/// 为冲突书名生成“书名（恢复）”“书名（恢复 2）”等可用名称。
pub async fn unique_title_tx(
    tx: &mut Transaction<'_, MySql>,
    base_title: &str,
    exclude_id: u64,
) -> Result<String, sqlx::Error> {
    for index in 1..=99u32 {
        let suffix = if index == 1 {
            "（恢复）".to_string()
        } else {
            format!("（恢复 {index}）")
        };
        let candidate = append_restore_suffix(base_title, &suffix);
        if !title_taken_tx(tx, &candidate, exclude_id).await? {
            return Ok(candidate);
        }
    }
    Ok(append_restore_suffix(base_title, "（恢复 99）"))
}

pub async fn restore_tx(
    tx: &mut Transaction<'_, MySql>,
    id: u64,
    title: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE books
           SET title = ?, deleted_at = NULL, deleted_by = NULL, updated_at = UTC_TIMESTAMP(3)
         WHERE id = ? AND deleted_at IS NOT NULL
        "#,
    )
    .bind(title)
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected())
}

pub async fn count_active_jobs_tx(
    tx: &mut Transaction<'_, MySql>,
    book_id: u64,
) -> Result<u64, sqlx::Error> {
    let count = sqlx::query_scalar::<_, u64>(
        "SELECT CAST(COUNT(*) AS UNSIGNED) FROM jobs WHERE book_id = ? AND status IN ('queued', 'running')",
    )
    .bind(book_id)
    .fetch_one(&mut **tx)
    .await?;
    Ok(count)
}

pub async fn collect_file_ids_tx(
    tx: &mut Transaction<'_, MySql>,
    book_id: u64,
) -> Result<Vec<u64>, sqlx::Error> {
    sqlx::query_scalar::<_, u64>(
        r#"
        SELECT cover_file_id AS file_id
          FROM books
         WHERE id = ? AND cover_file_id IS NOT NULL
        UNION
        SELECT source_file_id AS file_id
          FROM books
         WHERE id = ? AND source_file_id IS NOT NULL
        UNION
        SELECT audio_file_id AS file_id
          FROM chapters
         WHERE book_id = ? AND audio_file_id IS NOT NULL
        UNION
        SELECT s.audio_file_id AS file_id
          FROM segments s
          JOIN chapters c ON c.id = s.chapter_id
         WHERE c.book_id = ? AND s.audio_file_id IS NOT NULL
        UNION
        SELECT file_id
          FROM exports
         WHERE book_id = ? AND file_id IS NOT NULL
        "#,
    )
    .bind(book_id)
    .bind(book_id)
    .bind(book_id)
    .bind(book_id)
    .bind(book_id)
    .fetch_all(&mut **tx)
    .await
}

pub async fn delete_jobs_tx(
    tx: &mut Transaction<'_, MySql>,
    book_id: u64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM jobs
         WHERE book_id = ?
            OR chapter_id IN (SELECT id FROM chapters WHERE book_id = ?)
        "#,
    )
    .bind(book_id)
    .bind(book_id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected())
}

pub async fn delete_book_tx(tx: &mut Transaction<'_, MySql>, id: u64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM books WHERE id = ? AND deleted_at IS NOT NULL")
        .bind(id)
        .execute(&mut **tx)
        .await?;
    Ok(result.rows_affected())
}

async fn referenced_in_column_tx(
    tx: &mut Transaction<'_, MySql>,
    table: &'static str,
    column: &'static str,
    file_ids: &[u64],
) -> Result<Vec<u64>, sqlx::Error> {
    let mut qb =
        QueryBuilder::<MySql>::new(format!("SELECT {column} FROM {table} WHERE {column} IN ("));
    {
        let mut separated = qb.separated(", ");
        for file_id in file_ids {
            separated.push_bind(*file_id);
        }
    }
    qb.push(")");
    qb.build_query_scalar::<u64>().fetch_all(&mut **tx).await
}

/// 返回仍被任何业务数据引用的文件 ID；这些文件不能随书籍彻底删除。
pub async fn referenced_file_ids_tx(
    tx: &mut Transaction<'_, MySql>,
    file_ids: &[u64],
) -> Result<Vec<u64>, sqlx::Error> {
    if file_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut referenced = Vec::new();
    for (table, column) in [
        ("books", "cover_file_id"),
        ("books", "source_file_id"),
        ("chapters", "audio_file_id"),
        ("segments", "audio_file_id"),
        ("voices", "sample_file_id"),
        ("exports", "file_id"),
    ] {
        referenced.extend(referenced_in_column_tx(tx, table, column, file_ids).await?);
    }
    referenced.sort_unstable();
    referenced.dedup();
    Ok(referenced)
}

pub async fn file_objects_tx(
    tx: &mut Transaction<'_, MySql>,
    file_ids: &[u64],
) -> Result<Vec<FileObjectRow>, sqlx::Error> {
    if file_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut qb = QueryBuilder::<MySql>::new("SELECT id, object_key FROM files WHERE id IN (");
    {
        let mut separated = qb.separated(", ");
        for file_id in file_ids {
            separated.push_bind(*file_id);
        }
    }
    qb.push(")");
    qb.build_query_as::<FileObjectRow>()
        .fetch_all(&mut **tx)
        .await
}

pub async fn delete_files_tx(
    tx: &mut Transaction<'_, MySql>,
    file_ids: &[u64],
) -> Result<u64, sqlx::Error> {
    if file_ids.is_empty() {
        return Ok(0);
    }

    let mut qb = QueryBuilder::<MySql>::new("DELETE FROM files WHERE id IN (");
    {
        let mut separated = qb.separated(", ");
        for file_id in file_ids {
            separated.push_bind(*file_id);
        }
    }
    qb.push(")");
    let result = qb.build().execute(&mut **tx).await?;
    Ok(result.rows_affected())
}

pub async fn list_trash_ids(pool: &MySqlPool) -> Result<Vec<u64>, sqlx::Error> {
    sqlx::query_scalar::<_, u64>(
        "SELECT id FROM books WHERE deleted_at IS NOT NULL ORDER BY deleted_at ASC, id ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_expired_ids(
    pool: &MySqlPool,
    cutoff: NaiveDateTime,
    limit: u64,
) -> Result<Vec<u64>, sqlx::Error> {
    sqlx::query_scalar::<_, u64>(
        r#"
        SELECT id FROM books
         WHERE deleted_at IS NOT NULL AND deleted_at < ?
         ORDER BY deleted_at ASC
         LIMIT ?
        "#,
    )
    .bind(cutoff)
    .bind(limit as i64)
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_suffix_keeps_title_within_column_limit() {
        let title = "书".repeat(260);
        let result = append_restore_suffix(&title, "（恢复）");
        assert_eq!(result.chars().count(), 255);
        assert!(result.ends_with("（恢复）"));
    }

    #[test]
    fn restore_suffix_handles_empty_title() {
        assert_eq!(append_restore_suffix("   ", "（恢复）"), "未命名（恢复）");
    }
}
