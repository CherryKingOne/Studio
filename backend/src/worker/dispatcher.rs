use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Semaphore;

use crate::runtime::state::WorkerState;
use crate::shared::signal;
use crate::task_center::model::Job;
use crate::task_center::queue as jobs_queue;
use crate::task_center::repo as jobs_repo;
use crate::text_import::execute as imports_service;

pub async fn run(state: Arc<WorkerState>) -> anyhow::Result<()> {
    let concurrency = state.app.config.worker.concurrency;
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut reclaim_tick = tokio::time::interval(Duration::from_secs(30));
    let mut recycle_tick = tokio::time::interval(Duration::from_secs(3600));
    let shutdown = signal::shutdown_signal();
    tokio::pin!(shutdown);

    tracing::info!(
        worker_id = %state.worker_id,
        concurrency,
        "novelcast worker dispatcher started"
    );

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                tracing::info!("worker shutdown signal received");
                break;
            }
            _ = reclaim_tick.tick() => {
                match jobs_queue::reclaim_stale(&state.app.db, state.app.config.worker.stale_secs).await {
                    Ok(count) if count > 0 => tracing::warn!(count, "reclaimed stale jobs"),
                    Ok(_) => {}
                    Err(error) => tracing::error!(%error, "reclaim stale jobs failed"),
                }
            }
            _ = recycle_tick.tick() => {
                let cleanup_state = state.clone();
                tokio::spawn(async move {
                    match crate::recycle_bin::service::purge_expired(&cleanup_state.app).await {
                        Ok(count) if count > 0 => {
                            tracing::info!(count, "purged expired recycle bin books");
                        }
                        Ok(_) => {}
                        Err(error) => {
                            tracing::error!(%error, "purge expired recycle bin books failed");
                        }
                    }
                });
            }
            permit = semaphore.clone().acquire_owned() => {
                let permit = permit?;
                match jobs_queue::claim_next(&state.app.db, &state.worker_id).await {
                    Ok(Some(job)) => {
                        let state = state.clone();
                        tokio::spawn(async move {
                            handle_job(state, job).await;
                            drop(permit);
                        });
                    }
                    Ok(None) => {
                        drop(permit);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    Err(error) => {
                        drop(permit);
                        tracing::error!(%error, "claim job failed");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }

    let _ = tokio::time::timeout(
        Duration::from_secs(30),
        semaphore.acquire_many(concurrency as u32),
    )
    .await;
    Ok(())
}

async fn handle_job(state: Arc<WorkerState>, job: Job) {
    let job_id = job.id;
    let worker_id = state.worker_id.clone();
    let heartbeat_state = state.clone();
    let heartbeat_worker = worker_id.clone();

    let heartbeat = tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(
            heartbeat_state.app.config.worker.heartbeat_secs,
        ));
        loop {
            tick.tick().await;
            match jobs_queue::heartbeat(&heartbeat_state.app.db, job_id, &heartbeat_worker).await {
                Ok(true) => {}
                Ok(false) => {
                    tracing::warn!(job_id, "job heartbeat lost, stopping");
                    break;
                }
                Err(error) => {
                    tracing::error!(job_id, %error, "job heartbeat failed");
                    break;
                }
            }
        }
    });

    tracing::info!(job_id, job_type = %job.job_type, "job started");

    let outcome = match job.job_type.as_str() {
        "import_parse" => imports_service::run_import(&state.app, &job)
            .await
            .map(Some),
        "character_detect" => crate::casting::characters::manage::run_detect(&state.app, &job)
            .await
            .map(Some),
        "chapter_tts" => crate::speech::synthesize::run_tts(&state.app, &job)
            .await
            .map(|result| serde_json::to_value(result).ok())
            .map_err(|error| error.to_string()),
        "chapter_merge" => crate::audio::merge_chapter::run_merge(&state.app, &job)
            .await
            .map(Some),
        "book_export" => crate::delivery::exports::service::run_export(&state.app, &job)
            .await
            .map(|result| serde_json::to_value(result).ok())
            .map_err(|error| error.to_string()),
        other => Err(format!("未实现的任务类型: {other}")),
    };

    let completed = match outcome {
        Ok(result) => {
            jobs_queue::complete(&state.app.db, job_id, "succeeded", result.as_ref(), None).await
        }
        Err(error) => {
            if job.job_type == "book_export" {
                if let Some(export_id) = job
                    .payload
                    .0
                    .get("export_id")
                    .and_then(serde_json::Value::as_u64)
                {
                    crate::delivery::exports::service::mark_failed(&state.app, export_id, &error)
                        .await;
                }
            }
            let canceled = jobs_repo::is_cancel_requested(&state.app.db, job_id)
                .await
                .unwrap_or(false);
            let status = if canceled { "canceled" } else { "failed" };
            jobs_queue::complete(&state.app.db, job_id, status, None, Some(&error)).await
        }
    };

    if let Err(error) = completed {
        tracing::error!(job_id, %error, "complete job failed");
    }
    heartbeat.abort();
    tracing::info!(job_id, "job finished");
}
