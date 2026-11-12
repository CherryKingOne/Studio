use chrono::{Duration, NaiveDateTime, Utc};
use serde_json::Value;
use sqlx::types::Json;
use sqlx::{MySql, MySqlPool, QueryBuilder, Transaction};

use crate::features::jobs::model::{Job, JobItem};

pub struct NewJob {
    pub job_type: String,
    pub priority: u8,
    pub book_id: Option<u64>,
    pub chapter_id: Option<u64>,
    pub created_by: Option<u64>,
    pub payload: Value,
    pub total_items: u32,
    pub max_attempts: u8,
}

pub struct NewJobItem {
    pub item_type: String,
    pub item_id: u64,
}

const INSERT_JOB_SQL: &str = r#"
INSERT INTO jobs (job_type, status, priority, book_id, chapter_id, created_by, payload, total_items, max_attempts)
VALUES (?, 'queued', ?, ?, ?, ?, ?, ?, ?)
"#;

pub async fn insert(pool: &MySqlPool, job: &NewJob) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(INSERT_JOB_SQL)
        .bind(&job.job_type)
        .bind(job.priority)
        .bind(job.book_id)
        .bind(job.chapter_id)
        .bind(job.created_by)
        .bind(Json(job.payload.clone()))
        .bind(job.total_items)
        .bind(job.max_attempts)
        .execute(pool)
        .await?;
    Ok(result.last_insert_id())
}

pub async fn insert_tx(tx: &mut Transaction<'_, MySql>, job: &NewJob) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(INSERT_JOB_SQL)
        .bind(&job.job_type)
        .bind(job.priority)
        .bind(job.book_id)
        .bind(job.chapter_id)
        .bind(job.created_by)
        .bind(Json(job.payload.clone()))
        .bind(job.total_items)
        .bind(job.max_attempts)
        .execute(&mut **tx)
        .await?;
    Ok(result.last_insert_id())
}

pub async fn insert_items_tx(
    tx: &mut Transaction<'_, MySql>,
    job_id: u64,
    items: &[NewJobItem],
) -> Result<(), sqlx::Error> {
    for item in items {
        sqlx::query(
            "INSERT INTO job_items (job_id, item_type, item_id, status) VALUES (?, ?, ?, 'pending')",
        )
        .bind(job_id)
        .bind(&item.item_type)
        .bind(item.item_id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub async fn find_by_id(pool: &MySqlPool, id: u64) -> Result<Option<Job>, sqlx::Error> {
    sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn list(
    pool: &MySqlPool,
    query: &crate::features::jobs::dto::JobListQuery,
) -> Result<(Vec<Job>, u64), sqlx::Error> {
    let mut qb = QueryBuilder::<MySql>::new("SELECT * FROM jobs WHERE 1 = 1");
    let mut count_qb = QueryBuilder::<MySql>::new("SELECT COUNT(*) FROM jobs WHERE 1 = 1");

    if let Some(book_id) = query.book_id {
        qb.push(" AND book_id = ").push_bind(book_id);
        count_qb.push(" AND book_id = ").push_bind(book_id);
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
    if let Some(job_type) = query
        .job_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        qb.push(" AND job_type = ").push_bind(job_type);
        count_qb.push(" AND job_type = ").push_bind(job_type);
    }

    qb.push(" ORDER BY created_at DESC LIMIT ")
        .push_bind(query.page_size() as i64)
        .push(" OFFSET ")
        .push_bind(query.offset() as i64);

    let jobs = qb.build_query_as::<Job>().fetch_all(pool).await?;
    let total = count_qb.build_query_scalar::<u64>().fetch_one(pool).await?;
    Ok((jobs, total))
}

pub async fn items(pool: &MySqlPool, job_id: u64, limit: u64) -> Result<Vec<JobItem>, sqlx::Error> {
    sqlx::query_as::<_, JobItem>("SELECT * FROM job_items WHERE job_id = ? ORDER BY id ASC LIMIT ?")
        .bind(job_id)
        .bind(limit as i64)
        .fetch_all(pool)
        .await
}

pub async fn claim_next(pool: &MySqlPool, worker_id: &str) -> Result<Option<Job>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let job = sqlx::query_as::<_, Job>(
        r#"
        SELECT * FROM jobs
         WHERE status = 'queued' AND scheduled_at <= UTC_TIMESTAMP(3)
         ORDER BY priority DESC, created_at ASC
         LIMIT 1
         FOR UPDATE SKIP LOCKED
        "#,
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(mut job) = job else {
        tx.commit().await?;
        return Ok(None);
    };

    sqlx::query(
        r#"
        UPDATE jobs
           SET status = 'running', worker_id = ?, started_at = UTC_TIMESTAMP(3),
               heartbeat_at = UTC_TIMESTAMP(3), attempts = attempts + 1,
               updated_at = UTC_TIMESTAMP(3)
         WHERE id = ? AND status = 'queued'
        "#,
    )
    .bind(worker_id)
    .bind(job.id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    job.status = "running".to_string();
    job.worker_id = Some(worker_id.to_string());
    job.attempts = job.attempts.saturating_add(1);
    Ok(Some(job))
}

pub async fn heartbeat(
    pool: &MySqlPool,
    job_id: u64,
    worker_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE jobs SET heartbeat_at = UTC_TIMESTAMP(3) WHERE id = ? AND status = 'running' AND worker_id = ?",
    )
    .bind(job_id)
    .bind(worker_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_progress(
    pool: &MySqlPool,
    job_id: u64,
    progress: u8,
    done_items: u32,
    failed_items: u32,
    current_step: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE jobs
           SET progress = ?, done_items = ?, failed_items = ?, current_step = ?, updated_at = UTC_TIMESTAMP(3)
         WHERE id = ? AND status = 'running'
        "#,
    )
    .bind(progress)
    .bind(done_items)
    .bind(failed_items)
    .bind(current_step)
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_total_items(
    pool: &MySqlPool,
    job_id: u64,
    total_items: u32,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE jobs SET total_items = ? WHERE id = ?")
        .bind(total_items)
        .bind(job_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn complete(
    pool: &MySqlPool,
    job_id: u64,
    status: &str,
    result: Option<&Value>,
    error_message: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE jobs
           SET status = ?, result = ?, error_message = ?, progress = 100,
               current_step = NULL, finished_at = UTC_TIMESTAMP(3),
               worker_id = NULL, updated_at = UTC_TIMESTAMP(3)
         WHERE id = ?
        "#,
    )
    .bind(status)
    .bind(result.map(|value| Json(value.clone())))
    .bind(error_message)
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn request_cancel(pool: &MySqlPool, job_id: u64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE jobs SET cancel_requested = 1, updated_at = UTC_TIMESTAMP(3) WHERE id = ? AND status IN ('queued', 'running')",
    )
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn cancel_queued(pool: &MySqlPool, job_id: u64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE jobs SET status = 'canceled', finished_at = UTC_TIMESTAMP(3), updated_at = UTC_TIMESTAMP(3) WHERE id = ? AND status = 'queued'",
    )
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn is_cancel_requested(pool: &MySqlPool, job_id: u64) -> Result<bool, sqlx::Error> {
    let value = sqlx::query_scalar::<_, i8>("SELECT cancel_requested FROM jobs WHERE id = ?")
        .bind(job_id)
        .fetch_optional(pool)
        .await?;
    Ok(value.unwrap_or(0) != 0)
}

pub async fn reclaim_stale(pool: &MySqlPool, stale_secs: u64) -> Result<u64, sqlx::Error> {
    let cutoff = Utc::now().naive_utc() - Duration::seconds(stale_secs as i64);
    let result = sqlx::query(
        r#"
        UPDATE jobs
           SET status = IF(attempts >= max_attempts, 'failed', 'queued'),
               worker_id = NULL,
               error_message = IF(attempts >= max_attempts, 'worker heartbeat timeout', error_message),
               updated_at = UTC_TIMESTAMP(3)
         WHERE status = 'running' AND heartbeat_at < ?
        "#,
    )
    .bind(cutoff)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn count_active(
    pool: &MySqlPool,
    book_id: u64,
    job_type: &str,
) -> Result<u64, sqlx::Error> {
    let count = sqlx::query_scalar::<_, u64>(
        "SELECT COUNT(*) FROM jobs WHERE book_id = ? AND job_type = ? AND status IN ('queued', 'running')",
    )
    .bind(book_id)
    .bind(job_type)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn active_job_id(
    pool: &MySqlPool,
    book_id: u64,
    job_type: &str,
) -> Result<Option<u64>, sqlx::Error> {
    sqlx::query_scalar::<_, u64>(
        "SELECT id FROM jobs WHERE book_id = ? AND job_type = ? AND status IN ('queued', 'running') ORDER BY created_at ASC LIMIT 1",
    )
    .bind(book_id)
    .bind(job_type)
    .fetch_optional(pool)
    .await
}

pub async fn active_chapter_job_id(
    pool: &MySqlPool,
    chapter_id: u64,
    job_type: &str,
) -> Result<Option<u64>, sqlx::Error> {
    sqlx::query_scalar::<_, u64>(
        "SELECT id FROM jobs WHERE chapter_id = ? AND job_type = ? AND status IN ('queued', 'running') ORDER BY created_at ASC LIMIT 1",
    )
    .bind(chapter_id)
    .bind(job_type)
    .fetch_optional(pool)
    .await
}

pub async fn failed_item_ids(pool: &MySqlPool, job_id: u64) -> Result<Vec<u64>, sqlx::Error> {
    sqlx::query_scalar::<_, u64>(
        "SELECT item_id FROM job_items WHERE job_id = ? AND status = 'failed'",
    )
    .bind(job_id)
    .fetch_all(pool)
    .await
}

pub async fn first_failed_error(
    pool: &MySqlPool,
    job_id: u64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT error_message FROM job_items WHERE job_id = ? AND status = 'failed' AND error_message IS NOT NULL LIMIT 1",
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await
}

pub fn heartbeat_cutoff(stale_secs: u64) -> NaiveDateTime {
    Utc::now().naive_utc() - Duration::seconds(stale_secs as i64)
}
