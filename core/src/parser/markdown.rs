use crate::buffer::Document;
use crate::model::LineKind;

use super::shared::parsed_line;

pub(super) fn parse(document: &Document) -> Vec<crate::model::ParsedLine> {
    let mut parsed = Vec::with_capacity(document.line_count());
    let mut in_fenced_code_block = false;

    for raw in document.lines() {
        let (kind, heading_level) = classify_line(raw, in_fenced_code_block);
        if matches!(kind, LineKind::MarkdownCodeFence) {
            in_fenced_code_block = !in_fenced_code_block;
        }

        parsed.push(parsed_line(raw, kind, heading_level));
    }

    parsed
}

fn classify_line(raw: &str, in_fenced_code_block: bool) -> (LineKind, Option<u8>) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (LineKind::Empty, None);
    }

    if is_code_fence(trimmed) {
        return (LineKind::MarkdownCodeFence, None);
    }

    if in_fenced_code_block {
        return (LineKind::MarkdownCode, None);
    }

    if let Some(level) = heading_level(trimmed) {
        return (LineKind::MarkdownHeading, Some(level));
    }

    if is_list_item(trimmed) {
        return (LineKind::MarkdownListItem, None);
    }

    if is_quote(trimmed) {
        return (LineKind::MarkdownQuote, None);
    }

    if is_rule(trimmed) {
        return (LineKind::MarkdownRule, None);
    }

    (LineKind::MarkdownParagraph, None)
}

fn is_code_fence(line: &str) -> bool {
    line.starts_with("```") || line.starts_with("~~~")
}

fn heading_level(line: &str) -> Option<u8> {
    let hashes = line.chars().take_while(|ch| *ch == '#').count();
    (1..=6).contains(&hashes).then_some(hashes as u8)
}

fn is_list_item(line: &str) -> bool {
    is_unordered_list_item(line) || is_ordered_list_item(line)
}

fn is_unordered_list_item(line: &str) -> bool {
    let mut chars = line.chars();
    let Some(marker) = chars.next() else {
        return false;
    };
    if !matches!(marker, '-' | '*' | '+') {
        return false;
    }

    chars.next().is_some_and(char::is_whitespace)
}

fn is_ordered_list_item(line: &str) -> bool {
    let mut digits = 0usize;
    for ch in line.chars() {
        if ch.is_ascii_digit() {
            digits += 1;
        } else {
            break;
        }
    }
    if digits == 0 {
        return false;
    }

    let mut chars = line.chars().skip(digits);
    if chars.next() != Some('.') {
        return false;
    }

    chars.next().is_some_and(char::is_whitespace)
}

fn is_quote(line: &str) -> bool {
    line.starts_with('>')
}

fn is_rule(line: &str) -> bool {
    let compact = line.replace(' ', "");
    let bytes = compact.as_bytes();
    bytes.len() >= 3
        && (bytes.iter().all(|byte| *byte == b'-')
            || bytes.iter().all(|byte| *byte == b'*')
            || bytes.iter().all(|byte| *byte == b'_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_basic_markdown_lines() {
        let doc = Document::from_text(
            "# Heading\n- item\n> quote\n---\n```rs\nlet x = 1;\n```\nParagraph\n",
        );
        let parsed = parse(&doc);

        assert_eq!(parsed[0].kind, LineKind::MarkdownHeading);
        assert_eq!(parsed[0].markdown_heading_level, Some(1));
        assert_eq!(parsed[1].kind, LineKind::MarkdownListItem);
        assert_eq!(parsed[2].kind, LineKind::MarkdownQuote);
        assert_eq!(parsed[3].kind, LineKind::MarkdownRule);
        assert_eq!(parsed[4].kind, LineKind::MarkdownCodeFence);
        assert_eq!(parsed[5].kind, LineKind::MarkdownCode);
        assert_eq!(parsed[6].kind, LineKind::MarkdownCodeFence);
        assert_eq!(parsed[7].kind, LineKind::MarkdownParagraph);
    }

    #[test]
    fn captures_hash_heading_level_without_space() {
        let doc = Document::from_text("###Heading");
        let parsed = parse(&doc);

        assert_eq!(parsed[0].kind, LineKind::MarkdownHeading);
        assert_eq!(parsed[0].markdown_heading_level, Some(3));
    }

    #[test]
    fn classifies_markdown_checklist_and_spaced_ordered_item() {
        let doc = Document::from_text("- [x] done\n1.\titem");
        let parsed = parse(&doc);

        assert_eq!(parsed[0].kind, LineKind::MarkdownListItem);
        assert_eq!(parsed[1].kind, LineKind::MarkdownListItem);
    }
}
