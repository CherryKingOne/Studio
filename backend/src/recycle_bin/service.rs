use std::collections::HashSet;

use chrono::{Duration, Utc};

use crate::library::books::book_data;
use crate::recycle_bin::dto::{
    BatchItemResult, BatchPurgeRequest, BatchPurgeResponse, BatchRestoreRequest,
    BatchRestoreResponse, EmptyRecycleBinQuery, EmptyRecycleBinResponse, RecycleBinItem,
    RecycleBinListQuery, RecycleBinSummary, ResourceRef, ResourceTypeCount, RestoreBookRequest,
    RestoreResponse, RESOURCE_TYPE_BOOK,
};
use crate::recycle_bin::repo;
use crate::runtime::error::{AppError, FieldError};
use crate::runtime::state::AppState;
use crate::shared::pagination::Page;
use crate::shared::time;

const MAX_BATCH_ITEMS: usize = 100;
const AUTO_PURGE_BATCH: u64 = 100;
const AUTO_PURGE_ROUNDS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConflictPolicy {
    Fail,
    Rename,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PurgeOutcome {
    pub files_deleted: u64,
    pub objects_failed: u64,
}

pub async fn list(
    state: &AppState,
    query: RecycleBinListQuery,
) -> Result<Page<RecycleBinItem>, AppError> {
    let (rows, total) = repo::list(&state.db, &query).await?;
    let retention_days = state.config.recycle_bin.retention_days;
    let now = Utc::now().naive_utc();
    let items = rows
        .into_iter()
        .map(|row| RecycleBinItem::from_row(row, retention_days, now))
        .collect();

    Ok(Page::new(items, query.page(), query.page_size(), total))
}

pub async fn summary(state: &AppState) -> Result<RecycleBinSummary, AppError> {
    let row = repo::summary(&state.db).await?;
    let retention_days = state.config.recycle_bin.retention_days;
    let next_expire_at = if retention_days == 0 {
        None
    } else {
        row.oldest_deleted_at
            .map(|deleted_at| deleted_at + Duration::days(retention_days as i64))
    };

    Ok(RecycleBinSummary {
        total: row.total,
        resource_types: vec![ResourceTypeCount {
            resource_type: RESOURCE_TYPE_BOOK.to_string(),
            count: row.total,
        }],
        retention_days,
        oldest_deleted_at: time::iso_opt(row.oldest_deleted_at),
        next_expire_at: next_expire_at.map(time::iso),
    })
}

pub async fn restore(
    state: &AppState,
    resource_type: &str,
    id: u64,
    request: RestoreBookRequest,
) -> Result<RestoreResponse, AppError> {
    ensure_book_type(resource_type)?;
    let policy = parse_on_conflict(request.on_conflict.as_deref())?;
    restore_one(state, id, request.title, policy).await
}

pub async fn purge(
    state: &AppState,
    resource_type: &str,
    id: u64,
) -> Result<PurgeOutcome, AppError> {
    ensure_book_type(resource_type)?;
    purge_one(state, id).await
}

pub async fn batch_restore(
    state: &AppState,
    request: BatchRestoreRequest,
) -> Result<BatchRestoreResponse, AppError> {
    validate_items(&request.items)?;
    let policy = parse_on_conflict(request.on_conflict.as_deref())?;

    let mut restored = 0u64;
    let mut items = Vec::with_capacity(request.items.len());
    for item in request.items {
        match restore_one(state, item.id, None, policy).await {
            Ok(response) => {
                restored += 1;
                items.push(BatchItemResult {
                    resource_type: item.resource_type,
                    id: item.id,
                    success: true,
                    title: Some(response.title),
                    code: None,
                    message: None,
                });
            }
            Err(error) => items.push(failed_item(&item, &error)),
        }
    }

    Ok(BatchRestoreResponse {
        restored,
        failed: items.len() as u64 - restored,
        items,
    })
}

pub async fn batch_purge(
    state: &AppState,
    request: BatchPurgeRequest,
) -> Result<BatchPurgeResponse, AppError> {
    validate_items(&request.items)?;

    let mut purged = 0u64;
    let mut files_deleted = 0u64;
    let mut objects_failed = 0u64;
    let mut items = Vec::with_capacity(request.items.len());
    for item in request.items {
        match purge_one(state, item.id).await {
            Ok(outcome) => {
                purged += 1;
                files_deleted += outcome.files_deleted;
                objects_failed += outcome.objects_failed;
                items.push(BatchItemResult {
                    resource_type: item.resource_type,
                    id: item.id,
                    success: true,
                    title: None,
                    code: None,
                    message: None,
                });
            }
            Err(error) => items.push(failed_item(&item, &error)),
        }
    }

    Ok(BatchPurgeResponse {
        purged,
        failed: items.len() as u64 - purged,
        files_deleted,
        objects_failed,
        items,
    })
}

pub async fn empty(
    state: &AppState,
    query: EmptyRecycleBinQuery,
) -> Result<EmptyRecycleBinResponse, AppError> {
    if query.confirm != Some(true) {
        return Err(validation(
            "confirm",
            "必须传入 confirm=true 才能清空回收站",
        ));
    }

    let ids = repo::list_trash_ids(&state.db).await?;
    let mut purged = 0u64;
    let mut failed = 0u64;
    let mut files_deleted = 0u64;
    let mut objects_failed = 0u64;
    for id in ids {
        match purge_one(state, id).await {
            Ok(outcome) => {
                purged += 1;
                files_deleted += outcome.files_deleted;
                objects_failed += outcome.objects_failed;
            }
            Err(error) => {
                failed += 1;
                tracing::warn!(book_id = id, %error, "清空回收站时删除书籍失败");
            }
        }
    }

    Ok(EmptyRecycleBinResponse {
        purged,
        failed,
        files_deleted,
        objects_failed,
    })
}

/// Worker 定期调用：彻底删除超过保留期的回收站书籍。
pub async fn purge_expired(state: &AppState) -> Result<u64, AppError> {
    let retention_days = state.config.recycle_bin.retention_days;
    if retention_days == 0 {
        return Ok(0);
    }

    let cutoff = Utc::now().naive_utc() - Duration::days(retention_days as i64);
    let mut purged = 0u64;
    for _ in 0..AUTO_PURGE_ROUNDS {
        let ids = repo::list_expired_ids(&state.db, cutoff, AUTO_PURGE_BATCH).await?;
        if ids.is_empty() {
            break;
        }

        let mut round_purged = 0u64;
        for id in ids {
            match purge_one(state, id).await {
                Ok(_) => {
                    purged += 1;
                    round_purged += 1;
                }
                Err(error) => {
                    tracing::warn!(book_id = id, %error, "回收站过期自动清理失败");
                }
            }
        }
        if round_purged == 0 {
            break;
        }
    }

    Ok(purged)
}

async fn restore_one(
    state: &AppState,
    id: u64,
    requested_title: Option<String>,
    policy: ConflictPolicy,
) -> Result<RestoreResponse, AppError> {
    let requested_title = match requested_title {
        Some(title) => Some(validate_title(&title)?),
        None => None,
    };

    let mut tx = state.db.begin().await?;
    let book = repo::find_in_trash_for_update(&mut tx, id)
        .await?
        .ok_or(AppError::NotFound {
            resource: "回收站中的书籍",
        })?;

    let mut title = requested_title
        .clone()
        .unwrap_or_else(|| book.title.clone());
    if repo::title_taken_tx(&mut tx, &title, id).await? {
        title = match (requested_title.is_some(), policy) {
            (true, _) => {
                return Err(AppError::Duplicate(format!(
                    "书名「{title}」已被其他书籍占用"
                )));
            }
            (false, ConflictPolicy::Fail) => {
                return Err(AppError::Duplicate(format!(
                    "同名书籍「{title}」已存在，请指定新书名或设置 on_conflict=rename"
                )));
            }
            (false, ConflictPolicy::Rename) => repo::unique_title_tx(&mut tx, &title, id).await?,
        };
        if repo::title_taken_tx(&mut tx, &title, id).await? {
            return Err(AppError::Duplicate(
                "无法为恢复的书籍生成可用书名".to_string(),
            ));
        }
    }

    if repo::restore_tx(&mut tx, id, &title).await? == 0 {
        return Err(AppError::NotFound {
            resource: "回收站中的书籍",
        });
    }
    tx.commit().await?;

    let book = book_data::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound { resource: "书籍" })?;

    Ok(RestoreResponse {
        resource_type: RESOURCE_TYPE_BOOK.to_string(),
        id: book.id,
        book_id: book.id,
        title: book.title,
        status: book.status,
        restored_at: time::iso(book.updated_at),
    })
}

async fn purge_one(state: &AppState, id: u64) -> Result<PurgeOutcome, AppError> {
    let mut tx = state.db.begin().await?;
    let book = repo::find_in_trash_for_update(&mut tx, id).await?;
    if book.is_none() {
        let existing = repo::find_any_tx(&mut tx, id).await?;
        return Err(if existing.is_some() {
            AppError::InvalidState("书籍不在回收站中，无法彻底删除".to_string())
        } else {
            AppError::NotFound {
                resource: "回收站中的书籍",
            }
        });
    }

    if repo::count_active_jobs_tx(&mut tx, id).await? > 0 {
        return Err(AppError::InvalidState(
            "书籍存在进行中的任务，暂不能彻底删除".to_string(),
        ));
    }

    let file_ids = repo::collect_file_ids_tx(&mut tx, id).await?;
    repo::delete_jobs_tx(&mut tx, id).await?;
    if repo::delete_book_tx(&mut tx, id).await? == 0 {
        return Err(AppError::NotFound {
            resource: "回收站中的书籍",
        });
    }

    let referenced: HashSet<u64> = repo::referenced_file_ids_tx(&mut tx, &file_ids)
        .await?
        .into_iter()
        .collect();
    let deletable: Vec<u64> = file_ids
        .into_iter()
        .filter(|file_id| !referenced.contains(file_id))
        .collect();
    let objects = repo::file_objects_tx(&mut tx, &deletable).await?;
    let files_deleted = repo::delete_files_tx(&mut tx, &deletable).await?;
    tx.commit().await?;

    let mut objects_failed = 0u64;
    for object in objects {
        if let Err(error) = state.storage.delete(&object.object_key).await {
            objects_failed += 1;
            tracing::warn!(
                file_id = object.id,
                object_key = %object.object_key,
                %error,
                "回收站彻底删除对象存储文件失败"
            );
        }
    }

    Ok(PurgeOutcome {
        files_deleted,
        objects_failed,
    })
}

fn failed_item(item: &ResourceRef, error: &AppError) -> BatchItemResult {
    BatchItemResult {
        resource_type: item.resource_type.clone(),
        id: item.id,
        success: false,
        title: None,
        code: Some(error.code()),
        message: Some(error.to_string()),
    }
}

fn validate_items(items: &[ResourceRef]) -> Result<(), AppError> {
    if items.is_empty() || items.len() > MAX_BATCH_ITEMS {
        return Err(validation(
            "items",
            &format!("批量操作数量必须在 1 到 {MAX_BATCH_ITEMS} 之间"),
        ));
    }

    let mut seen = HashSet::new();
    for item in items {
        ensure_book_type(&item.resource_type)?;
        if !seen.insert(item.id) {
            return Err(validation("items", "存在重复的书籍 ID"));
        }
    }
    Ok(())
}

fn ensure_book_type(resource_type: &str) -> Result<(), AppError> {
    if resource_type == RESOURCE_TYPE_BOOK {
        Ok(())
    } else {
        Err(validation("resource_type", "暂只支持 book（书籍）"))
    }
}

fn parse_on_conflict(value: Option<&str>) -> Result<ConflictPolicy, AppError> {
    match value.map(str::trim).unwrap_or("fail") {
        "fail" => Ok(ConflictPolicy::Fail),
        "rename" => Ok(ConflictPolicy::Rename),
        _ => Err(validation("on_conflict", "只能是 fail 或 rename")),
    }
}

fn validate_title(title: &str) -> Result<String, AppError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(validation("title", "书名不能为空"));
    }
    if title.chars().count() > 255 {
        return Err(validation("title", "书名不能超过 255 个字符"));
    }
    Ok(title.to_string())
}

fn validation(field: &str, message: &str) -> AppError {
    AppError::Validation(vec![FieldError {
        field: field.to_string(),
        message: message.to_string(),
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_conflict_defaults_to_fail() {
        assert_eq!(
            parse_on_conflict(None).expect("default"),
            ConflictPolicy::Fail
        );
        assert_eq!(
            parse_on_conflict(Some("rename")).expect("rename"),
            ConflictPolicy::Rename
        );
        assert!(parse_on_conflict(Some("other")).is_err());
    }

    #[test]
    fn resource_type_rejects_non_book() {
        assert!(ensure_book_type(RESOURCE_TYPE_BOOK).is_ok());
        assert!(ensure_book_type("chapter").is_err());
    }

    #[test]
    fn batch_items_reject_duplicates() {
        let items = vec![
            ResourceRef {
                resource_type: "book".to_string(),
                id: 1,
            },
            ResourceRef {
                resource_type: "book".to_string(),
                id: 1,
            },
        ];
        assert!(validate_items(&items).is_err());
    }
}
