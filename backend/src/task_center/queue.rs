use sqlx::{MySql, MySqlPool, Transaction};

use crate::runtime::error::AppError;
use crate::task_center::model::Job;
use crate::task_center::repo::{self, NewJob, NewJobItem};

pub async fn enqueue(
    pool: &MySqlPool,
    job: &NewJob,
    items: &[NewJobItem],
) -> Result<u64, AppError> {
    let mut tx = pool.begin().await?;
    let job_id = repo::insert_tx(&mut tx, job).await?;
    repo::insert_items_tx(&mut tx, job_id, items).await?;
    tx.commit().await?;
    Ok(job_id)
}

pub async fn enqueue_tx(
    tx: &mut Transaction<'_, MySql>,
    job: &NewJob,
    items: &[NewJobItem],
) -> Result<u64, AppError> {
    let job_id = repo::insert_tx(tx, job).await?;
    repo::insert_items_tx(tx, job_id, items).await?;
    Ok(job_id)
}

pub async fn claim_next(pool: &MySqlPool, worker_id: &str) -> Result<Option<Job>, AppError> {
    Ok(repo::claim_next(pool, worker_id).await?)
}

pub async fn heartbeat(pool: &MySqlPool, job_id: u64, worker_id: &str) -> Result<bool, AppError> {
    Ok(repo::heartbeat(pool, job_id, worker_id).await?)
}

pub async fn reclaim_stale(pool: &MySqlPool, stale_secs: u64) -> Result<u64, AppError> {
    Ok(repo::reclaim_stale(pool, stale_secs).await?)
}

pub async fn complete(
    pool: &MySqlPool,
    job_id: u64,
    status: &str,
    result: Option<&serde_json::Value>,
    error_message: Option<&str>,
) -> Result<(), AppError> {
    repo::complete(pool, job_id, status, result, error_message).await?;
    Ok(())
}
