use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::recycle_bin::repo::DeletedBookRow;
use crate::shared::time;

/// 回收站当前只接受书籍；章节、段落、角色随书籍一起进入回收站。
pub const RESOURCE_TYPE_BOOK: &str = "book";

#[derive(Debug, Deserialize)]
pub struct RecycleBinListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub keyword: Option<String>,
}

impl RecycleBinListQuery {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn page_size(&self) -> u64 {
        self.page_size.unwrap_or(20).clamp(1, 100)
    }

    pub fn offset(&self) -> u64 {
        (self.page() - 1) * self.page_size()
    }
}

#[derive(Debug, Serialize)]
pub struct DeletedByDto {
    pub id: u64,
    pub username: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RecycleBinItem {
    pub resource_type: String,
    pub id: u64,
    pub title: String,
    pub author: Option<String>,
    pub cover_url: Option<String>,
    pub status: String,
    pub total_chapters: u32,
    pub total_segments: u32,
    pub total_duration_ms: u64,
    pub deleted_at: String,
    pub deleted_by: Option<DeletedByDto>,
    pub expires_at: Option<String>,
    pub remaining_days: Option<i64>,
}

impl RecycleBinItem {
    pub fn from_row(row: DeletedBookRow, retention_days: u64, now: NaiveDateTime) -> Self {
        let DeletedBookRow {
            book,
            deleted_by_username,
            deleted_by_display_name,
        } = row;

        let deleted_at = book.deleted_at.unwrap_or(now);
        let expires_at = expires_at(deleted_at, retention_days);
        let remaining_days = expires_at.map(|value| remaining_days(value, now));
        let deleted_by = match (book.deleted_by, deleted_by_username) {
            (Some(id), Some(username)) => Some(DeletedByDto {
                id,
                username,
                display_name: deleted_by_display_name,
            }),
            _ => None,
        };

        Self {
            resource_type: RESOURCE_TYPE_BOOK.to_string(),
            id: book.id,
            title: book.title,
            author: book.author,
            cover_url: book.cover_file_id.map(|id| format!("/api/v1/files/{id}")),
            status: book.status,
            total_chapters: book.total_chapters,
            total_segments: book.total_segments,
            total_duration_ms: book.total_duration_ms,
            deleted_at: time::iso(deleted_at),
            deleted_by,
            expires_at: expires_at.map(time::iso),
            remaining_days,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ResourceTypeCount {
    pub resource_type: String,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct RecycleBinSummary {
    pub total: u64,
    pub resource_types: Vec<ResourceTypeCount>,
    pub retention_days: u64,
    pub oldest_deleted_at: Option<String>,
    pub next_expire_at: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RestoreBookRequest {
    /// 恢复时可以指定新书名；冲突时也可配合 `on_conflict=rename` 自动改名。
    pub title: Option<String>,
    /// `fail`（默认）或 `rename`。
    pub on_conflict: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceRef {
    pub resource_type: String,
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct BatchRestoreRequest {
    pub items: Vec<ResourceRef>,
    pub on_conflict: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchPurgeRequest {
    pub items: Vec<ResourceRef>,
}

#[derive(Debug, Serialize)]
pub struct RestoreResponse {
    pub resource_type: String,
    pub id: u64,
    pub book_id: u64,
    pub title: String,
    pub status: String,
    pub restored_at: String,
}

#[derive(Debug, Serialize)]
pub struct BatchItemResult {
    pub resource_type: String,
    pub id: u64,
    pub success: bool,
    pub title: Option<String>,
    pub code: Option<u32>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchRestoreResponse {
    pub restored: u64,
    pub failed: u64,
    pub items: Vec<BatchItemResult>,
}

#[derive(Debug, Serialize)]
pub struct BatchPurgeResponse {
    pub purged: u64,
    pub failed: u64,
    pub files_deleted: u64,
    pub objects_failed: u64,
    pub items: Vec<BatchItemResult>,
}

#[derive(Debug, Serialize)]
pub struct PurgeResponse {
    pub resource_type: String,
    pub id: u64,
    pub purged: bool,
    pub files_deleted: u64,
    pub objects_failed: u64,
}

#[derive(Debug, Deserialize)]
pub struct EmptyRecycleBinQuery {
    pub confirm: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct EmptyRecycleBinResponse {
    pub purged: u64,
    pub failed: u64,
    pub files_deleted: u64,
    pub objects_failed: u64,
}

pub(crate) fn expires_at(deleted_at: NaiveDateTime, retention_days: u64) -> Option<NaiveDateTime> {
    if retention_days == 0 {
        None
    } else {
        Some(deleted_at + Duration::days(retention_days as i64))
    }
}

/// 剩余天数向上取整：还有 1 秒也算 1 天，已过期返回 0。
fn remaining_days(expires_at: NaiveDateTime, now: NaiveDateTime) -> i64 {
    let seconds = (expires_at - now).num_seconds();
    if seconds <= 0 {
        0
    } else {
        (seconds + 86_399) / 86_400
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn dt(day: u32, hour: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, day)
            .expect("valid date")
            .and_hms_opt(hour, 0, 0)
            .expect("valid time")
    }

    #[test]
    fn retention_zero_means_keep_forever() {
        assert_eq!(expires_at(dt(1, 0), 0), None);
    }

    #[test]
    fn expires_at_adds_retention_days() {
        assert_eq!(
            expires_at(dt(1, 0), 30),
            Some(dt(1, 0) + Duration::days(30))
        );
    }

    #[test]
    fn remaining_days_rounds_up_partial_days() {
        let expires = dt(3, 0);
        assert_eq!(remaining_days(expires, dt(2, 12)), 1);
        assert_eq!(remaining_days(expires, dt(2, 0)), 1);
        assert_eq!(remaining_days(expires, dt(1, 0)), 2);
    }

    #[test]
    fn remaining_days_is_zero_when_expired() {
        let expires = dt(2, 0);
        assert_eq!(remaining_days(expires, dt(2, 0)), 0);
        assert_eq!(remaining_days(expires, dt(3, 0)), 0);
    }
}
