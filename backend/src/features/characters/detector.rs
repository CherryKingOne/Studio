use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use regex::Regex;

use crate::features::characters::model::Character;
use crate::features::segments::model::Segment;

const STOPWORDS: &[&str] = &[
    "他们", "她们", "大家", "众人", "有人", "那人", "这人", "男人", "女人", "老人", "孩子", "少年",
    "少女", "声音", "心中", "心里", "嘴角", "目光", "自己", "对方", "两人", "三人",
];

#[derive(Debug, Clone)]
pub struct SpeakerGuess {
    pub character_id: Option<u64>,
    pub name: Option<String>,
    pub confidence: f32,
    pub pattern: &'static str,
}

impl SpeakerGuess {
    pub fn none() -> Self {
        Self {
            character_id: None,
            name: None,
            confidence: 0.0,
            pattern: "none",
        }
    }
}

pub fn collect_candidates(segments: &[Segment]) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for segment in segments
        .iter()
        .filter(|segment| segment.seg_type == "narration")
    {
        for captures in name_verb_regex().captures_iter(&segment.text) {
            if let Some(name) = captures.get(1) {
                let name = name.as_str();
                if is_valid_name(name) {
                    *counts.entry(name.to_string()).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

pub fn guess_speaker(segments: &[Segment], index: usize, characters: &[Character]) -> SpeakerGuess {
    let Some(segment) = segments.get(index) else {
        return SpeakerGuess::none();
    };
    if segment.seg_type != "dialogue" {
        return SpeakerGuess::none();
    }

    let aliases = build_alias_map(characters);
    let prev = if index > 0 && segments[index - 1].chapter_id == segment.chapter_id {
        Some(&segments[index - 1])
    } else {
        None
    };
    let next = if index + 1 < segments.len() && segments[index + 1].chapter_id == segment.chapter_id
    {
        Some(&segments[index + 1])
    } else {
        None
    };

    let mut candidates: Vec<(String, f32, &'static str)> = Vec::new();

    if let Some(next) = next.filter(|item| item.seg_type == "narration") {
        if let Some(name) = leading_name_with_verb(&next.text) {
            candidates.push((name, 0.95, "next_leading"));
        }
    }

    if let Some(prev) = prev.filter(|item| item.seg_type == "narration") {
        if let Some(name) = leading_name_with_verb(&prev.text) {
            candidates.push((name, 0.90, "prev_leading"));
        }
        if let Some(name) = leading_name_with_colon(&prev.text) {
            candidates.push((name, 0.85, "prev_colon"));
        }
        if let Some(name) = any_name_with_verb(&prev.text) {
            candidates.push((name, 0.75, "prev_context"));
        }
    }

    if candidates.is_empty() {
        if let Some(next) = next.filter(|item| item.seg_type == "narration") {
            if let Some(name) = any_name_with_verb(&next.text) {
                candidates.push((name, 0.75, "next_context"));
            }
        }
    }

    if let Some((name, confidence, pattern)) = candidates
        .into_iter()
        .max_by(|left, right| left.1.total_cmp(&right.1))
    {
        let character_id = aliases.get(&name).copied();
        return SpeakerGuess {
            character_id,
            name: Some(name),
            confidence,
            pattern,
        };
    }

    // 代词启发：只能给出候选，不自动绑定（置信度低于 0.6）。
    if let Some(next) = next.filter(|item| item.seg_type == "narration") {
        if pronoun_regex().is_match(next.text.trim()) {
            if let Some(previous) = segments[..index]
                .iter()
                .rev()
                .find(|item| item.seg_type == "dialogue" && item.speaker_id.is_some())
            {
                return SpeakerGuess {
                    character_id: previous.speaker_id,
                    name: previous.speaker_id.and_then(|speaker_id| {
                        characters
                            .iter()
                            .find(|character| character.id == speaker_id)
                            .map(|character| character.name.clone())
                    }),
                    confidence: 0.40,
                    pattern: "pronoun",
                };
            }
        }
    }

    SpeakerGuess::none()
}

fn build_alias_map(characters: &[Character]) -> HashMap<String, u64> {
    let mut aliases = HashMap::new();
    for character in characters {
        aliases.insert(character.name.clone(), character.id);
        for alias in &character.aliases.0 {
            aliases.insert(alias.clone(), character.id);
        }
    }
    aliases
}

fn is_valid_name(name: &str) -> bool {
    if !(2..=4).contains(&name.chars().count()) {
        return false;
    }
    let stopwords: HashSet<&str> = STOPWORDS.iter().copied().collect();
    !stopwords.contains(name)
}

fn leading_name_with_verb(text: &str) -> Option<String> {
    leading_name_verb_regex()
        .captures(text.trim())
        .and_then(|captures| captures.get(1))
        .map(|name| name.as_str().to_string())
        .filter(|name| is_valid_name(name))
}

fn leading_name_with_colon(text: &str) -> Option<String> {
    colon_regex()
        .captures(text.trim())
        .and_then(|captures| captures.get(1))
        .map(|name| name.as_str().to_string())
        .filter(|name| is_valid_name(name))
}

fn any_name_with_verb(text: &str) -> Option<String> {
    name_verb_regex()
        .captures_iter(text)
        .filter_map(|captures| captures.get(1))
        .map(|name| name.as_str().to_string())
        .find(|name| is_valid_name(name))
}

fn name_verb_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"([\u4e00-\u9fff]{2,4}?)(?:说道|说|道|问道|问|答道|答|喊道|叫道|笑道|冷笑道|低声道|喃喃道|开口|回答|追问|反问|嘀咕|嘟囔)",
        )
        .expect("valid name verb regex")
    })
}

fn leading_name_verb_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"^[\s“「『]*([\u4e00-\u9fff]{2,4}?)(?:说道|说|道|问道|问|答道|答|喊道|叫道|笑道|冷笑道|低声道|喃喃道|开口|回答|追问|反问|嘀咕|嘟囔)",
        )
        .expect("valid leading name verb regex")
    })
}

fn colon_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"^[\s“「『]*([\u4e00-\u9fff]{2,4}?)[：:]").expect("valid colon regex")
    })
}

fn pronoun_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^[，。！？、\s]*([他她])").expect("valid pronoun regex"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::types::Json;

    fn segment(
        id: u64,
        chapter_id: u64,
        idx: u32,
        text: &str,
        seg_type: &str,
        speaker_id: Option<u64>,
    ) -> Segment {
        Segment {
            id,
            chapter_id,
            idx,
            text: text.to_string(),
            char_count: text.chars().count() as u32,
            seg_type: seg_type.to_string(),
            speaker_id,
            speaker_confidence: None,
            voice_id: None,
            emotion: "neutral".to_string(),
            rate_percent: 0,
            pitch_hz: 0,
            volume_percent: 0,
            status: "pending".to_string(),
            audio_file_id: None,
            audio_duration_ms: None,
            cache_key: None,
            error_message: None,
            attempts: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }

    fn character(id: u64, name: &str) -> Character {
        Character {
            id,
            book_id: 1,
            name: name.to_string(),
            aliases: Json(Vec::new()),
            gender: "unknown".to_string(),
            age_group: "unknown".to_string(),
            description: None,
            default_voice_id: None,
            color: None,
            source: "manual".to_string(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }

    #[test]
    fn detects_leading_attribution() {
        let segments = vec![
            segment(1, 1, 1, "夜色很浓。", "narration", None),
            segment(2, 1, 2, "阿澜说道：", "narration", None),
            segment(3, 1, 3, "走吧。", "dialogue", None),
        ];
        let characters = vec![character(7, "阿澜")];
        let guess = guess_speaker(&segments, 2, &characters);
        assert_eq!(guess.character_id, Some(7));
        assert!(guess.confidence >= 0.6);
    }

    #[test]
    fn collects_repeated_candidates() {
        let segments = vec![
            segment(1, 1, 1, "阿澜说道。", "narration", None),
            segment(2, 1, 2, "阿澜笑道。", "narration", None),
            segment(3, 1, 3, "阿澜问道。", "narration", None),
        ];
        let candidates = collect_candidates(&segments);
        assert_eq!(candidates.get("阿澜").copied(), Some(3));
    }
}
