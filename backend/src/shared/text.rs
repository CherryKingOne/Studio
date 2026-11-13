use std::sync::OnceLock;

use encoding_rs::GB18030;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    Utf8,
    Utf8Bom,
    Gb18030,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChapter {
    pub index: u32,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSegment {
    pub text: String,
    pub seg_type: &'static str,
}

pub fn decode_text(bytes: &[u8]) -> Result<(String, TextEncoding), String> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let text =
            std::str::from_utf8(&bytes[3..]).map_err(|_| "UTF-8 BOM 文件解码失败".to_string())?;
        return Ok((text.to_string(), TextEncoding::Utf8Bom));
    }

    if let Ok(text) = std::str::from_utf8(bytes) {
        return Ok((text.to_string(), TextEncoding::Utf8));
    }

    let (decoded, _, had_errors) = GB18030.decode(bytes);
    let text = decoded.into_owned();
    let total = text.chars().count();
    let replacements = text.chars().filter(|ch| *ch == '\u{FFFD}').count();

    if total == 0 || replacements * 1000 > total.saturating_mul(5) {
        return Err("文件编码不受支持，请转换为 UTF-8 后重试".to_string());
    }
    if had_errors {
        tracing::warn!("GB18030 decode had errors, replacement ratio is acceptable");
    }

    Ok((text, TextEncoding::Gb18030))
}

pub fn normalize_text(text: &str) -> String {
    let mut normalized = text.replace("\r\n", "\n").replace('\r', "\n");

    for zero_width in ['\u{200B}', '\u{200C}', '\u{200D}', '\u{FEFF}'] {
        normalized = normalized.replace(zero_width, "");
    }

    let mut lines = Vec::new();
    for line in normalized.split('\n') {
        lines.push(line.replace('\u{3000}', " ").trim_end().to_string());
    }

    let joined = lines.join("\n");
    let mut result = String::with_capacity(joined.len());
    let mut newline_count = 0usize;
    for ch in joined.chars() {
        if ch == '\n' {
            newline_count += 1;
            if newline_count <= 2 {
                result.push(ch);
            }
        } else {
            newline_count = 0;
            result.push(ch);
        }
    }

    result.trim().to_string()
}

pub fn split_chapters(text: &str) -> Vec<TextChapter> {
    let normalized = normalize_text(text);
    if normalized.is_empty() {
        return Vec::new();
    }

    let lines: Vec<&str> = normalized.lines().collect();
    let mut marks = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if chapter_title_regex().is_match(trimmed) {
            marks.push((index, trimmed.to_string()));
        }
    }

    let mut chapters = Vec::new();
    if marks.len() >= 2 {
        if let Some((first_index, _)) = marks.first() {
            let preface = lines[..*first_index].join("\n");
            if !preface.trim().is_empty() {
                chapters.push(format!("前言\n{}", preface.trim()));
            }
        }

        for (position, (line_index, title)) in marks.iter().enumerate() {
            let end = marks
                .get(position + 1)
                .map(|(index, _)| *index)
                .unwrap_or(lines.len());
            let body = lines[line_index + 1..end].join("\n");
            if !body.trim().is_empty() {
                chapters.push(format!("{title}\n{}", body.trim()));
            }
        }

        return build_chapters(chapters);
    }

    build_chapters(fallback_chapters(&lines))
}

pub fn split_segments(text: &str, max_chars: usize) -> Vec<TextSegment> {
    let max_chars = max_chars.clamp(50, 1000);
    let normalized = normalize_text(text);
    let mut segments: Vec<TextSegment> = Vec::new();

    for line in normalized.lines() {
        let paragraph = line.trim();
        if paragraph.is_empty() {
            continue;
        }

        for (chunk, seg_type) in split_quoted_chunks(paragraph) {
            for part in split_long_text(&chunk, max_chars) {
                if part.is_empty() {
                    continue;
                }
                segments.push(TextSegment {
                    text: part,
                    seg_type,
                });
            }
        }
    }

    merge_tiny_segments(segments, max_chars)
}

pub fn sanitize_filename(name: &str) -> String {
    let mut cleaned: String = name
        .chars()
        .map(|ch| {
            if matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || ch.is_control()
            {
                '_'
            } else {
                ch
            }
        })
        .collect();
    cleaned = cleaned.trim().to_string();
    if cleaned.chars().count() > 80 {
        cleaned = cleaned.chars().take(80).collect();
    }
    if cleaned.is_empty() {
        "untitled".to_string()
    } else {
        cleaned
    }
}

fn build_chapters(raw_chapters: Vec<String>) -> Vec<TextChapter> {
    raw_chapters
        .into_iter()
        .filter_map(|raw| {
            let mut lines = raw.lines();
            let title = lines.next()?.trim().to_string();
            let body = lines.collect::<Vec<_>>().join("\n").trim().to_string();
            if body.is_empty() {
                return None;
            }
            Some((title, body))
        })
        .enumerate()
        .map(|(index, (title, body))| TextChapter {
            index: index as u32 + 1,
            title,
            body,
        })
        .collect()
}

fn fallback_chapters(lines: &[&str]) -> Vec<String> {
    const CHUNK_CHARS: usize = 30_000;
    let mut chapters = Vec::new();
    let mut buffer = String::new();

    for line in lines {
        if buffer.chars().count() + line.chars().count() > CHUNK_CHARS && !buffer.trim().is_empty()
        {
            chapters.push(format!("第 {} 部分\n{}", chapters.len() + 1, buffer.trim()));
            buffer.clear();
        }
        buffer.push_str(line);
        buffer.push('\n');
    }

    if !buffer.trim().is_empty() {
        chapters.push(format!("第 {} 部分\n{}", chapters.len() + 1, buffer.trim()));
    }
    chapters
}

fn split_quoted_chunks(paragraph: &str) -> Vec<(String, &'static str)> {
    let mut chunks: Vec<(String, &'static str)> = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;

    for ch in paragraph.chars() {
        match ch {
            '“' | '「' | '『' => {
                if in_quote {
                    current.push(ch);
                } else {
                    push_chunk(&mut chunks, &mut current, "narration");
                    in_quote = true;
                }
            }
            '”' | '」' | '』' => {
                if in_quote {
                    push_chunk(&mut chunks, &mut current, "dialogue");
                    in_quote = false;
                } else {
                    current.push(ch);
                }
            }
            _ => current.push(ch),
        }
    }

    push_chunk(
        &mut chunks,
        &mut current,
        if in_quote { "dialogue" } else { "narration" },
    );
    chunks
}

fn push_chunk(
    chunks: &mut Vec<(String, &'static str)>,
    current: &mut String,
    seg_type: &'static str,
) {
    let text = current.trim().to_string();
    current.clear();
    if !text.is_empty() {
        chunks.push((text, seg_type));
    }
}

fn split_long_text(text: &str, max_chars: usize) -> Vec<String> {
    if text.chars().count() <= max_chars {
        return vec![text.trim().to_string()];
    }

    let mut sentences = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '。' | '！' | '？' | '!' | '?' | '…') {
            sentences.push(std::mem::take(&mut current));
        }
    }
    if !current.trim().is_empty() {
        sentences.push(current);
    }

    let mut chunks = Vec::new();
    let mut buffer = String::new();
    for sentence in sentences {
        let sentence_len = sentence.chars().count();
        if sentence_len > max_chars {
            if !buffer.trim().is_empty() {
                chunks.push(std::mem::take(&mut buffer).trim().to_string());
            }
            let mut part = String::new();
            for ch in sentence.chars() {
                part.push(ch);
                if part.chars().count() >= max_chars {
                    chunks.push(std::mem::take(&mut part).trim().to_string());
                }
            }
            if !part.trim().is_empty() {
                chunks.push(part.trim().to_string());
            }
        } else if buffer.chars().count() + sentence_len > max_chars {
            if !buffer.trim().is_empty() {
                chunks.push(std::mem::take(&mut buffer).trim().to_string());
            }
            buffer = sentence;
        } else {
            buffer.push_str(&sentence);
        }
    }

    if !buffer.trim().is_empty() {
        chunks.push(buffer.trim().to_string());
    }
    chunks
}

fn merge_tiny_segments(segments: Vec<TextSegment>, max_chars: usize) -> Vec<TextSegment> {
    let mut merged: Vec<TextSegment> = Vec::new();

    for segment in segments {
        if let Some(last) = merged.last_mut() {
            let last_len = last.text.chars().count();
            let segment_len = segment.text.chars().count();
            if last.seg_type == segment.seg_type
                && (last_len < 4 || segment_len < 4)
                && last_len + segment_len <= max_chars
            {
                last.text.push_str(&segment.text);
                continue;
            }
        }
        merged.push(segment);
    }

    merged
}

fn chapter_title_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"^\s*(?:第[零一二三四五六七八九十百千万两0-9]+[章回节卷篇集部][\s:：、.．\-]*.{0,40}|(?:序章|楔子|引子|前言|后记|尾声|番外)(?:[\s:：、.．\-]+.{0,40})?)\s*$",
        )
        .expect("valid chapter title regex")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_utf8_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice("第一章 山雨".as_bytes());
        let (text, encoding) = decode_text(&bytes).expect("decode");
        assert_eq!(text, "第一章 山雨");
        assert_eq!(encoding, TextEncoding::Utf8Bom);
    }

    #[test]
    fn decodes_gb18030() {
        let (bytes, _, _) = GB18030.encode("第一章 山雨");
        let (text, encoding) = decode_text(bytes.as_ref()).expect("decode");
        assert_eq!(text, "第一章 山雨");
        assert_eq!(encoding, TextEncoding::Gb18030);
    }

    #[test]
    fn splits_chapters_with_titles() {
        let text = "前言内容\n\n第一章 山雨\n山中下雨了。\n\n第二章 入山\n他走进山中。\n";
        let chapters = split_chapters(text);
        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters[0].title, "前言");
        assert_eq!(chapters[1].title, "第一章 山雨");
        assert_eq!(chapters[1].body, "山中下雨了。");
    }

    #[test]
    fn splits_dialogue_and_narration() {
        let text = "夜色很浓。\n“你来了。”他说。\n「走吧。」";
        let segments = split_segments(text, 300);
        assert_eq!(segments[0].text, "夜色很浓。");
        assert_eq!(segments[0].seg_type, "narration");
        assert!(segments
            .iter()
            .any(|segment| segment.seg_type == "dialogue"));
        assert!(segments.iter().all(|segment| !segment.text.contains('“')));
    }

    #[test]
    fn splits_long_paragraph() {
        let text = "这是一个很长的句子。".repeat(100);
        let segments = split_segments(&text, 50);
        assert!(segments
            .iter()
            .all(|segment| segment.text.chars().count() <= 50));
        assert!(segments.len() > 1);
    }
}
