use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Semaphore;

use crate::common::signal;
use crate::features::imports::service as imports_service;
use crate::features::jobs::model::Job;
use crate::features::jobs::queue as jobs_queue;
use crate::features::jobs::repo as jobs_repo;
use crate::state::WorkerState;

pub async fn run(state: Arc<WorkerState>) -> anyhow::Result<()> {
    let concurrency = state.app.config.worker.concurrency;
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut reclaim_tick = tokio::time::interval(Duration::from_secs(30));
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
        "character_detect" => crate::features::characters::service::run_detect(&state.app, &job)
            .await
            .map(Some),
        other => Err(format!("未实现的任务类型: {other}")),
    };

    let completed = match outcome {
        Ok(result) => {
            jobs_queue::complete(&state.app.db, job_id, "succeeded", result.as_ref(), None).await
        }
        Err(error) => {
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
