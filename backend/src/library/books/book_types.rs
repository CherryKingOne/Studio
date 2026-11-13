use serde::{Deserialize, Serialize};

use crate::library::books::book_data::Book;

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
            created_at: crate::shared::time::iso(book.created_at),
            updated_at: crate::shared::time::iso(book.updated_at),
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
    pub settings: BookSettings,
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
            settings: serde_json::from_value(book.settings.0).unwrap_or_default(),
            status: book.status,
            total_chapters: book.total_chapters,
            total_segments: book.total_segments,
            total_duration_ms: book.total_duration_ms,
            error_message: book.error_message,
            created_at: crate::shared::time::iso(book.created_at),
            updated_at: crate::shared::time::iso(book.updated_at),
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

#[derive(Debug, Serialize)]
pub struct DeleteBookResponse {
    pub id: u64,
    pub title: String,
    pub deleted: bool,
    pub deleted_at: String,
    pub deleted_by: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::{json, Value};
    use sqlx::types::Json;

    fn sample_book(settings: Value) -> Book {
        Book {
            id: 2,
            title: "示例小说".to_string(),
            author: Some("咸鱼老白".to_string()),
            description: None,
            language: "zh-CN".to_string(),
            cover_file_id: None,
            source_file_id: None,
            default_narrator_voice_id: None,
            settings: Json(settings),
            status: "draft".to_string(),
            total_chapters: 0,
            total_segments: 0,
            total_duration_ms: 0,
            created_by: Some(1),
            error_message: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
            deleted_by: None,
        }
    }

    fn keys(value: &Value) -> Vec<String> {
        value
            .as_object()
            .expect("json object")
            .keys()
            .cloned()
            .collect()
    }

    #[test]
    fn book_detail_field_order_matches_api_doc() {
        let detail = BookDetail::from(sample_book(json!({})));
        let value = serde_json::to_value(&detail).expect("serialize book detail");

        assert_eq!(
            keys(&value),
            vec![
                "id",
                "title",
                "author",
                "description",
                "language",
                "cover_file_id",
                "cover_url",
                "source_file_id",
                "default_narrator_voice_id",
                "settings",
                "status",
                "total_chapters",
                "total_segments",
                "total_duration_ms",
                "error_message",
                "created_at",
                "updated_at",
            ]
        );
    }

    #[test]
    fn settings_field_order_matches_api_doc() {
        let value = serde_json::to_value(BookSettings::default()).expect("serialize settings");
        assert_eq!(keys(&value), vec!["pause_ms", "loudness", "output", "tts"]);
        assert_eq!(
            keys(&value["pause_ms"]),
            vec!["narration", "dialogue", "chapter_end"]
        );
        assert_eq!(keys(&value["loudness"]), vec!["i", "tp", "lra"]);
        assert_eq!(
            keys(&value["output"]),
            vec!["format", "sample_rate", "channels", "bitrate_kbps"]
        );
        assert_eq!(keys(&value["tts"]), vec!["provider", "max_attempts"]);
    }

    #[test]
    fn settings_rebuild_from_legacy_json_keeps_doc_order() {
        let stored = json!({
            "tts": { "max_attempts": 3, "provider": "openai" },
            "output": { "bitrate_kbps": 128, "channels": 1, "format": "mp3", "sample_rate": 44100 },
            "loudness": { "i": -16.0, "lra": 11.0, "tp": -1.5 },
            "pause_ms": { "chapter_end": 1200, "dialogue": 250, "narration": 350 }
        });

        let detail = BookDetail::from(sample_book(stored));
        let value = serde_json::to_value(&detail).expect("serialize book detail");

        assert_eq!(
            keys(&value["settings"]),
            vec!["pause_ms", "loudness", "output", "tts"]
        );
    }

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
