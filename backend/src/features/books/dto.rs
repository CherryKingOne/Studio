use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::features::books::model::Book;

#[derive(Debug, Deserialize)]
pub struct BookListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub keyword: Option<String>,
    pub status: Option<String>,
    pub include_archived: Option<bool>,
}

impl BookListQuery {
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

#[derive(Debug, Deserialize)]
pub struct CreateBookRequest {
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBookRequest {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub default_narrator_voice_id: Option<u64>,
    pub settings: Option<BookSettings>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookSettings {
    #[serde(default)]
    pub pause_ms: PauseMs,
    #[serde(default)]
    pub loudness: Loudness,
    #[serde(default)]
    pub output: OutputSettings,
    #[serde(default)]
    pub tts: TtsSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PauseMs {
    pub narration: u32,
    pub dialogue: u32,
    pub chapter_end: u32,
}

impl Default for PauseMs {
    fn default() -> Self {
        Self {
            narration: 350,
            dialogue: 250,
            chapter_end: 1200,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loudness {
    pub i: f32,
    pub tp: f32,
    pub lra: f32,
}

impl Default for Loudness {
    fn default() -> Self {
        Self {
            i: -16.0,
            tp: -1.5,
            lra: 11.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSettings {
    pub format: String,
    pub sample_rate: u32,
    pub channels: u8,
    pub bitrate_kbps: u32,
}

impl Default for OutputSettings {
    fn default() -> Self {
        Self {
            format: "mp3".to_string(),
            sample_rate: 44_100,
            channels: 1,
            bitrate_kbps: 128,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsSettings {
    pub provider: String,
    pub max_attempts: u8,
}

impl Default for TtsSettings {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            max_attempts: 3,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BookSummary {
    pub id: u64,
    pub title: String,
    pub author: Option<String>,
    pub cover_url: Option<String>,
    pub status: String,
    pub total_chapters: u32,
    pub total_segments: u32,
    pub total_duration_ms: u64,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&Book> for BookSummary {
    fn from(book: &Book) -> Self {
        Self {
            id: book.id,
            title: book.title.clone(),
            author: book.author.clone(),
            cover_url: book.cover_file_id.map(|id| format!("/api/v1/files/{id}")),
            status: book.status.clone(),
            total_chapters: book.total_chapters,
            total_segments: book.total_segments,
            total_duration_ms: book.total_duration_ms,
            created_at: crate::common::time::iso(book.created_at),
            updated_at: crate::common::time::iso(book.updated_at),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BookDetail {
    pub id: u64,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub language: String,
    pub cover_file_id: Option<u64>,
    pub cover_url: Option<String>,
    pub source_file_id: Option<u64>,
    pub default_narrator_voice_id: Option<u64>,
    pub settings: Value,
    pub status: String,
    pub total_chapters: u32,
    pub total_segments: u32,
    pub total_duration_ms: u64,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Book> for BookDetail {
    fn from(book: Book) -> Self {
        Self {
            id: book.id,
            title: book.title,
            author: book.author,
            description: book.description,
            language: book.language,
            cover_file_id: book.cover_file_id,
            cover_url: book.cover_file_id.map(|id| format!("/api/v1/files/{id}")),
            source_file_id: book.source_file_id,
            default_narrator_voice_id: book.default_narrator_voice_id,
            settings: book.settings.0,
            status: book.status,
            total_chapters: book.total_chapters,
            total_segments: book.total_segments,
            total_duration_ms: book.total_duration_ms,
            error_message: book.error_message,
            created_at: crate::common::time::iso(book.created_at),
            updated_at: crate::common::time::iso(book.updated_at),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BookStats {
    pub total_chapters: u64,
    pub segmented_chapters: u64,
    pub tts_done_chapters: u64,
    pub merged_chapters: u64,
    pub total_segments: u64,
    pub done_segments: u64,
    pub failed_segments: u64,
    pub total_duration_ms: u64,
    pub active_job_count: u64,
}

#[derive(Debug, Serialize)]
pub struct CoverResponse {
    pub cover_file_id: u64,
    pub cover_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_match_spec() {
        let settings = BookSettings::default();
        assert_eq!(settings.pause_ms.narration, 350);
        assert_eq!(settings.pause_ms.dialogue, 250);
        assert_eq!(settings.pause_ms.chapter_end, 1200);
        assert_eq!(settings.output.sample_rate, 44_100);
        assert_eq!(settings.output.channels, 1);
        assert_eq!(settings.output.bitrate_kbps, 128);
        assert_eq!(settings.tts.provider, "openai");
    }
}
