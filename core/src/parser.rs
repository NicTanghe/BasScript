use crate::buffer::Document;
use crate::links::extract_script_links;
use crate::model::{DocumentFormat, LineKind, ParsedLine};

pub fn parse_document(document: &Document) -> Vec<ParsedLine> {
    parse_document_with_format(document, DocumentFormat::Fountain)
}

pub fn parse_document_with_format(document: &Document, format: DocumentFormat) -> Vec<ParsedLine> {
    match format {
        DocumentFormat::Fountain => parse_fountain(document),
        DocumentFormat::Markdown => parse_markdown(document),
    }
}

fn parse_fountain(document: &Document) -> Vec<ParsedLine> {
    let mut parsed = Vec::with_capacity(document.line_count());
    let mut previous_kind = LineKind::Empty;

    for raw in document.lines() {
        let kind = classify_line(raw, &previous_kind);
        previous_kind = kind.clone();

        parsed.push(ParsedLine {
            kind,
            raw: raw.clone(),
            script_links: extract_script_links(raw),
        });
    }

    parsed
}

fn parse_markdown(document: &Document) -> Vec<ParsedLine> {
    let mut parsed = Vec::with_capacity(document.line_count());
    let mut in_fenced_code_block = false;

    for raw in document.lines() {
        let kind = classify_markdown_line(raw, in_fenced_code_block);
        if matches!(kind, LineKind::MarkdownCodeFence) {
            in_fenced_code_block = !in_fenced_code_block;
        }

        parsed.push(ParsedLine {
            kind,
            raw: raw.clone(),
            script_links: extract_script_links(raw),
        });
    }

    parsed
}

fn classify_line(raw: &str, previous_kind: &LineKind) -> LineKind {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return LineKind::Empty;
    }

    if is_scene_heading(trimmed) {
        return LineKind::SceneHeading;
    }

    if is_transition(trimmed) {
        return LineKind::Transition;
    }

    if is_character(trimmed) {
        return LineKind::Character;
    }

    if is_parenthetical(trimmed)
        && matches!(
            previous_kind,
            LineKind::Character | LineKind::Dialogue | LineKind::Parenthetical
        )
    {
        return LineKind::Parenthetical;
    }

    if matches!(
        previous_kind,
        LineKind::Character | LineKind::Dialogue | LineKind::Parenthetical
    ) {
        return LineKind::Dialogue;
    }

    LineKind::Action
}

fn is_scene_heading(line: &str) -> bool {
    let upper = line.trim_start().to_uppercase();
    let starts_with_marker = ["INT.", "EXT.", "EST.", "INT/EXT.", "I/E."]
        .iter()
        .any(|prefix| upper.starts_with(prefix));

    starts_with_marker
}

fn is_transition(line: &str) -> bool {
    let upper = line.to_uppercase();
    upper.ends_with(" TO:")
        || upper == "CUT TO:"
        || upper == "FADE OUT."
        || upper == "FADE TO BLACK."
}

fn is_character(line: &str) -> bool {
    if line.chars().count() > 32 {
        return false;
    }

    let words = line.split_whitespace().count();
    if words == 0 || words > 4 {
        return false;
    }

    if !line
        .chars()
        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || " .()'-".contains(ch))
    {
        return false;
    }

    !line.ends_with(':')
}

fn is_parenthetical(line: &str) -> bool {
    line.starts_with('(') && line.ends_with(')')
}

fn classify_markdown_line(raw: &str, in_fenced_code_block: bool) -> LineKind {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return LineKind::Empty;
    }

    if is_markdown_code_fence(trimmed) {
        return LineKind::MarkdownCodeFence;
    }

    if in_fenced_code_block {
        return LineKind::MarkdownCode;
    }

    if is_markdown_heading(trimmed) {
        return LineKind::MarkdownHeading;
    }

    if is_markdown_list_item(trimmed) {
        return LineKind::MarkdownListItem;
    }

    if is_markdown_quote(trimmed) {
        return LineKind::MarkdownQuote;
    }

    if is_markdown_rule(trimmed) {
        return LineKind::MarkdownRule;
    }

    LineKind::MarkdownParagraph
}

fn is_markdown_code_fence(line: &str) -> bool {
    line.starts_with("```") || line.starts_with("~~~")
}

fn is_markdown_heading(line: &str) -> bool {
    let hashes = line.chars().take_while(|ch| *ch == '#').count();
    hashes > 0 && hashes <= 6
}

fn is_markdown_list_item(line: &str) -> bool {
    is_markdown_unordered_list_item(line) || is_markdown_ordered_list_item(line)
}

fn is_markdown_unordered_list_item(line: &str) -> bool {
    let mut chars = line.chars();
    let Some(marker) = chars.next() else {
        return false;
    };
    if !matches!(marker, '-' | '*' | '+') {
        return false;
    }

    chars.next().is_some_and(char::is_whitespace)
}

fn is_markdown_ordered_list_item(line: &str) -> bool {
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

fn is_markdown_quote(line: &str) -> bool {
    line.starts_with('>')
}

fn is_markdown_rule(line: &str) -> bool {
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
    fn classifies_basic_fountain_subset() {
        let doc = Document::from_text(
            "INT. COFFEE SHOP - DAY\n\nSARAH\n(smiling)\nIt is just text.\nCUT TO:\n",
        );

        let parsed = parse_document(&doc);

        assert_eq!(parsed[0].kind, LineKind::SceneHeading);
        assert_eq!(parsed[1].kind, LineKind::Empty);
        assert_eq!(parsed[2].kind, LineKind::Character);
        assert_eq!(parsed[3].kind, LineKind::Parenthetical);
        assert_eq!(parsed[4].kind, LineKind::Dialogue);
        assert_eq!(parsed[5].kind, LineKind::Transition);
    }

    #[test]
    fn classifies_mixed_case_scene_heading() {
        let doc = Document::from_text("Int. kitchen - day\nAction");
        let parsed = parse_document(&doc);

        assert_eq!(parsed[0].kind, LineKind::SceneHeading);
        assert_eq!(parsed[1].kind, LineKind::Action);
    }

    #[test]
    fn extracts_internal_script_links_per_line() {
        let doc =
            Document::from_text("He opens [door-kitchen-main] and [that door](door-kitchen-main).");
        let parsed = parse_document(&doc);

        assert_eq!(parsed[0].script_links.len(), 2);
        assert_eq!(parsed[0].script_links[0].target, "door-kitchen-main");
        assert_eq!(parsed[0].script_links[1].label, "that door");
    }

    #[test]
    fn classifies_basic_markdown_lines() {
        let doc = Document::from_text(
            "# Heading\n- item\n> quote\n---\n```rs\nlet x = 1;\n```\nParagraph\n",
        );
        let parsed = parse_document_with_format(&doc, DocumentFormat::Markdown);

        assert_eq!(parsed[0].kind, LineKind::MarkdownHeading);
        assert_eq!(parsed[1].kind, LineKind::MarkdownListItem);
        assert_eq!(parsed[2].kind, LineKind::MarkdownQuote);
        assert_eq!(parsed[3].kind, LineKind::MarkdownRule);
        assert_eq!(parsed[4].kind, LineKind::MarkdownCodeFence);
        assert_eq!(parsed[5].kind, LineKind::MarkdownCode);
        assert_eq!(parsed[6].kind, LineKind::MarkdownCodeFence);
        assert_eq!(parsed[7].kind, LineKind::MarkdownParagraph);
    }

    #[test]
    fn classifies_hash_heading_without_space() {
        let doc = Document::from_text("#Heading");
        let parsed = parse_document_with_format(&doc, DocumentFormat::Markdown);
        assert_eq!(parsed[0].kind, LineKind::MarkdownHeading);
    }

    #[test]
    fn classifies_markdown_checklist_and_spaced_ordered_item() {
        let doc = Document::from_text("- [x] done\n1.\titem");
        let parsed = parse_document_with_format(&doc, DocumentFormat::Markdown);
        assert_eq!(parsed[0].kind, LineKind::MarkdownListItem);
        assert_eq!(parsed[1].kind, LineKind::MarkdownListItem);
    }
}
