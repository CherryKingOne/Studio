use serde::{Deserialize, Serialize};

use crate::features::exports::model::ExportRow;

#[derive(Debug, Deserialize)]
pub struct CreateExportRequest {
    pub include_cover: Option<bool>,
    pub chapter_ids: Option<Vec<u64>>,
}

#[derive(Debug, Serialize)]
pub struct CreateExportResponse {
    pub export_id: u64,
    pub job_id: u64,
}

#[derive(Debug, Serialize)]
pub struct ExportDto {
    pub id: u64,
    pub book_id: u64,
    pub export_type: String,
    pub status: String,
    pub total_chapters: u32,
    pub included_chapters: u32,
    pub file_id: Option<u64>,
    pub download_url: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub finished_at: Option<String>,
}

impl From<ExportRow> for ExportDto {
    fn from(row: ExportRow) -> Self {
        Self {
            id: row.id,
            book_id: row.book_id,
            export_type: row.export_type,
            status: row.status,
            total_chapters: row.total_chapters,
            included_chapters: row.included_chapters,
            file_id: row.file_id,
            download_url: row
                .file_id
                .map(|_| format!("/api/v1/exports/{}/download", row.id)),
            error_message: row.error_message,
            created_at: crate::common::time::iso(row.created_at),
            finished_at: crate::common::time::iso_opt(row.finished_at),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportPayload {
    pub export_id: u64,
    pub book_id: u64,
    pub chapter_ids: Option<Vec<u64>>,
    pub include_cover: bool,
}

#[derive(Debug, Serialize)]
pub struct ExportRunResult {
    pub export_id: u64,
    pub file_id: u64,
    pub included_chapters: u32,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportOptions {
    pub include_cover: bool,
    pub chapter_ids: Option<Vec<u64>>,
}
