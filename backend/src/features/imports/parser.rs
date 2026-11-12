use crate::common::text::{decode_text, split_chapters, split_segments, TextEncoding, TextSegment};

#[derive(Debug)]
pub struct ParsedBook {
    pub encoding: TextEncoding,
    pub chapters: Vec<ParsedChapter>,
    pub total_segments: u32,
}

#[derive(Debug)]
pub struct ParsedChapter {
    pub index: u32,
    pub title: String,
    pub raw_text: String,
    pub char_count: u32,
    pub segments: Vec<TextSegment>,
}

pub fn parse_book(bytes: &[u8], max_chars: usize) -> Result<ParsedBook, String> {
    let (text, encoding) = decode_text(bytes)?;
    let chapters = split_chapters(&text);
    if chapters.is_empty() {
        return Err("未解析到任何章节，请检查文件内容".to_string());
    }
    if chapters.len() > 2000 {
        return Err("章节数超过 2000，请拆分文件后导入".to_string());
    }

    let mut parsed = Vec::new();
    let mut total_segments = 0u32;
    for chapter in chapters {
        let segments = split_segments(&chapter.body, max_chars);
        if segments.is_empty() {
            continue;
        }
        total_segments += segments.len() as u32;
        parsed.push(ParsedChapter {
            index: parsed.len() as u32 + 1,
            title: chapter.title,
            char_count: chapter.body.chars().count() as u32,
            raw_text: chapter.body,
            segments,
        });
    }

    if parsed.is_empty() {
        return Err("章节正文为空，无法导入".to_string());
    }

    Ok(ParsedBook {
        encoding,
        chapters: parsed,
        total_segments,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chapters_and_segments() {
        let text = "第一章 山雨\n夜色很浓。\n“你来了。”他说。\n\n第二章 入山\n他走进山中。\n";
        let book = parse_book(text.as_bytes(), 300).expect("parse book");
        assert_eq!(book.encoding, TextEncoding::Utf8);
        assert_eq!(book.chapters.len(), 2);
        assert_eq!(book.chapters[0].title, "第一章 山雨");
        assert!(book.chapters[0]
            .segments
            .iter()
            .any(|segment| segment.seg_type == "dialogue"));
        assert!(book.total_segments >= 3);
    }

    #[test]
    fn rejects_empty_text() {
        assert!(parse_book(b"", 300).is_err());
    }
}
