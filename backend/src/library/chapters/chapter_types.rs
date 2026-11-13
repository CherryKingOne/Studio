use serde::{Deserialize, Serialize};

use crate::library::chapters::chapter_data::{Chapter, ChapterRow};

#[derive(Debug, Deserialize)]
pub struct ChapterListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub status: Option<String>,
    pub keyword: Option<String>,
}

impl ChapterListQuery {
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
pub struct ChapterSummary {
    pub id: u64,
    pub index: u32,
    pub title: String,
    pub char_count: u32,
    pub status: String,
    pub audio_duration_ms: Option<u32>,
    pub tts_done_segments: u32,
    pub segment_count: u64,
    pub error_message: Option<String>,
    pub updated_at: String,
}

impl From<&ChapterRow> for ChapterSummary {
    fn from(row: &ChapterRow) -> Self {
        Self {
            id: row.chapter.id,
            index: row.chapter.idx,
            title: row.chapter.title.clone(),
            char_count: row.chapter.char_count,
            status: row.chapter.status.clone(),
            audio_duration_ms: row.chapter.audio_duration_ms,
            tts_done_segments: row.chapter.tts_done_segments,
            segment_count: row.segment_count,
            error_message: row.chapter.error_message.clone(),
            updated_at: crate::shared::time::iso(row.chapter.updated_at),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ChapterDetail {
    pub id: u64,
    pub book_id: u64,
    pub index: u32,
    pub title: String,
    pub raw_text: String,
    pub char_count: u32,
    pub status: String,
    pub narrator_voice_id: Option<u64>,
    pub audio_file_id: Option<u64>,
    pub audio_duration_ms: Option<u32>,
    pub audio_rev: u32,
    pub tts_done_segments: u32,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Chapter> for ChapterDetail {
    fn from(chapter: Chapter) -> Self {
        Self {
            id: chapter.id,
            book_id: chapter.book_id,
            index: chapter.idx,
            title: chapter.title,
            raw_text: chapter.raw_text,
            char_count: chapter.char_count,
            status: chapter.status,
            narrator_voice_id: chapter.narrator_voice_id,
            audio_file_id: chapter.audio_file_id,
            audio_duration_ms: chapter.audio_duration_ms,
            audio_rev: chapter.audio_rev,
            tts_done_segments: chapter.tts_done_segments,
            error_message: chapter.error_message,
            created_at: crate::shared::time::iso(chapter.created_at),
            updated_at: crate::shared::time::iso(chapter.updated_at),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateChapterRequest {
    pub title: Option<String>,
    pub raw_text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResegmentRequest {
    #[serde(default)]
    pub confirm: bool,
    pub max_chars: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ResegmentResponse {
    pub segments_created: u64,
    pub char_count: u32,
}
