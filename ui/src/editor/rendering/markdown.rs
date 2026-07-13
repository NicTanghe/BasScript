pub(crate) fn markdown_visual_text(
    parsed_line: &ParsedLine,
) -> Option<(usize, String, Option<bool>)> {
    markdown_visual_text_for_kind(
        &parsed_line.raw,
        &parsed_line.kind,
        parsed_line.markdown_heading_level,
    )
}

pub(crate) fn markdown_visual_text_for_kind(
    raw: &str,
    kind: &LineKind,
    _markdown_heading_level: Option<u8>,
) -> Option<(usize, String, Option<bool>)> {
    match kind {
        LineKind::MarkdownHeading => {
            let (consumed, rendered) = markdown_heading_visual(raw);
            Some((consumed, rendered, None))
        }
        LineKind::MarkdownListItem => Some(markdown_list_item_visual(raw)),
        LineKind::MarkdownQuote => {
            let (consumed, rendered) = markdown_quote_visual(raw);
            Some((consumed, rendered, None))
        }
        LineKind::MarkdownRule => Some((0, "────────────────────────".to_string(), None)),
        LineKind::MarkdownCodeFence => Some((0, "```".to_string(), None)),
        _ => None,
    }
}

pub(crate) fn markdown_render_override_for_raw(raw: &str) -> Option<ProcessedLineRenderOverride> {
    let parsed_line =
        parse_document_with_format(&Document::from_text(raw), DocumentFormat::Markdown)
            .into_iter()
            .next()?;
    if matches!(
        parsed_line.kind,
        LineKind::MarkdownParagraph | LineKind::Empty
    ) {
        return None;
    }

    Some(ProcessedLineRenderOverride {
        kind: parsed_line.kind,
        markdown_heading_level: parsed_line.markdown_heading_level,
    })
}

pub(crate) fn markdown_line_style(
    kind: &LineKind,
    markdown_heading_level: Option<u8>,
) -> Option<LineRenderStyle> {
    match kind {
        LineKind::MarkdownHeading => {
            Some(markdown_heading_style(markdown_heading_level.unwrap_or(1)))
        }
        LineKind::MarkdownListItem => Some(default_line_render_style()),
        LineKind::MarkdownQuote => Some(LineRenderStyle::new(
            FontVariant::Italic,
            COLOR_MARKDOWN_QUOTE,
            1.0,
            1.0,
        )),
        LineKind::MarkdownCodeFence => Some(LineRenderStyle::new(
            FontVariant::Bold,
            COLOR_MARKDOWN_CODE,
            1.0,
            1.0,
        )),
        LineKind::MarkdownCode => Some(LineRenderStyle::new(
            FontVariant::Regular,
            COLOR_MARKDOWN_CODE,
            1.0,
            1.0,
        )),
        LineKind::MarkdownRule => Some(LineRenderStyle::new(
            FontVariant::Bold,
            COLOR_MARKDOWN_RULE,
            1.0,
            1.0,
        )),
        LineKind::MarkdownParagraph => Some(default_line_render_style()),
        _ => None,
    }
}

pub(crate) fn markdown_heading_style(level: u8) -> LineRenderStyle {
    let (font_scale, line_height_scale) = match level.clamp(1, 6) {
        1 => (1.80, 2.15),
        2 => (1.55, 1.85),
        3 => (1.35, 1.60),
        4 => (1.20, 1.45),
        5 => (1.05, 1.30),
        _ => (0.95, 1.20),
    };

    LineRenderStyle::new(
        FontVariant::Bold,
        COLOR_MARKDOWN_HEADING,
        font_scale,
        line_height_scale,
    )
}

pub(crate) fn markdown_heading_visual(raw: &str) -> (usize, String) {
    let leading = leading_markdown_whitespace(raw);
    let trimmed = raw.chars().skip(leading).collect::<Vec<_>>();
    let mut hashes = 0usize;
    while trimmed.get(hashes).is_some_and(|ch| *ch == '#') {
        hashes += 1;
    }

    let mut consumed = hashes;
    if trimmed.get(consumed).is_some_and(|ch| *ch == ' ') {
        consumed += 1;
    }

    let text = trimmed[consumed..].iter().collect::<String>();
    (leading.saturating_add(consumed), text)
}

pub(crate) fn markdown_quote_visual(raw: &str) -> (usize, String) {
    let leading = leading_markdown_whitespace(raw);
    let trimmed = raw.chars().skip(leading).collect::<Vec<_>>();
    let mut consumed = 0usize;
    while trimmed.get(consumed).is_some_and(|ch| *ch == '>') {
        consumed += 1;
        if trimmed.get(consumed).is_some_and(|ch| *ch == ' ') {
            consumed += 1;
        }
    }
    let text = trimmed[consumed..].iter().collect::<String>();
    (leading.saturating_add(consumed), text)
}

pub(crate) fn markdown_list_item_visual(raw: &str) -> (usize, String, Option<bool>) {
    let leading = leading_markdown_whitespace(raw);
    let trimmed = raw.chars().skip(leading).collect::<Vec<_>>();
    if trimmed.is_empty() {
        return (leading, String::new(), None);
    }

    if let Some(marker_end) = unordered_list_content_start(&trimmed) {
        if let Some((consumed, checked, content_start)) =
            markdown_checklist_marker(&trimmed, marker_end)
        {
            let text = trimmed[content_start..].iter().collect::<String>();
            return (leading.saturating_add(consumed), text, Some(checked));
        }

        let text = trimmed[marker_end..].iter().collect::<String>();
        return (
            leading.saturating_add(marker_end),
            format!("• {text}"),
            None,
        );
    }

    if let Some((prefix, content_start)) = ordered_list_content_start(&trimmed) {
        if let Some((consumed, checked, checklist_content_start)) =
            markdown_checklist_marker(&trimmed, content_start)
        {
            let text = trimmed[checklist_content_start..]
                .iter()
                .collect::<String>();
            return (
                leading.saturating_add(consumed),
                format!("{prefix} {text}"),
                Some(checked),
            );
        }

        let text = trimmed[content_start..].iter().collect::<String>();
        return (
            leading.saturating_add(content_start),
            format!("{prefix} {text}"),
            None,
        );
    }

    (0, raw.to_string(), None)
}

pub(crate) fn markdown_checklist_marker(
    chars: &[char],
    start: usize,
) -> Option<(usize, bool, usize)> {
    let checked_char = *chars.get(start + 1)?;
    let checked = matches!(checked_char, 'x' | 'X');
    if chars.get(start).is_some_and(|ch| *ch == '[')
        && matches!(checked_char, 'x' | 'X' | ' ')
        && chars.get(start + 2).is_some_and(|ch| *ch == ']')
    {
        let mut content_start = start + 3;
        if chars.get(content_start).is_some_and(|ch| *ch == ' ') {
            content_start += 1;
        }
        return Some((content_start, checked, content_start));
    }

    None
}

pub(crate) fn leading_markdown_whitespace(raw: &str) -> usize {
    raw.chars()
        .take_while(|ch| matches!(*ch, ' ' | '\t'))
        .count()
}

pub(crate) fn unordered_list_content_start(chars: &[char]) -> Option<usize> {
    if chars.is_empty() || !matches!(chars[0], '-' | '*' | '+') {
        return None;
    }

    let mut index = 1usize;
    let mut saw_whitespace = false;
    while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
        saw_whitespace = true;
        index += 1;
    }

    saw_whitespace.then_some(index)
}

pub(crate) fn ordered_list_content_start(chars: &[char]) -> Option<(String, usize)> {
    let mut digits = 0usize;
    while chars.get(digits).is_some_and(|ch| ch.is_ascii_digit()) {
        digits += 1;
    }
    if digits == 0 || chars.get(digits) != Some(&'.') {
        return None;
    }

    let mut content_start = digits + 1;
    let mut saw_whitespace = false;
    while chars
        .get(content_start)
        .is_some_and(|ch| ch.is_whitespace())
    {
        saw_whitespace = true;
        content_start += 1;
    }
    if !saw_whitespace {
        return None;
    }

    let prefix = chars[..=digits].iter().collect::<String>();
    Some((prefix, content_start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_markdown_front_matter_fields() {
        let document = Document::from_text(
            "---\ntarget: door-kitchen-main\ntype: prop\nname: Kitchen Main Door\naliases: []\n---\nBody\n",
        );

        let front_matter =
            markdown_front_matter_display(&document).expect("front matter should be parsed");

        assert_eq!(front_matter.closing_line_index, 5);
        assert_eq!(front_matter.fields.target, "door-kitchen-main");
        assert_eq!(front_matter.fields.entity_type, "prop");
        assert_eq!(front_matter.fields.name, "Kitchen Main Door");
    }

    #[test]
    fn detects_markdown_heading_override_from_raw_line() {
        let render_override =
            markdown_render_override_for_raw("# test").expect("heading should be detected");

        assert_eq!(render_override.kind, LineKind::MarkdownHeading);
        assert_eq!(render_override.markdown_heading_level, Some(1));

        let (consumed, rendered, checklist) = markdown_visual_text_for_kind(
            "# test",
            &render_override.kind,
            render_override.markdown_heading_level,
        )
        .expect("heading should render as markdown");

        assert_eq!(consumed, 2);
        assert_eq!(rendered, "test");
        assert_eq!(checklist, None);
    }

    #[test]
    fn gives_markdown_headings_extra_line_height() {
        let h1 = markdown_heading_style(1);
        let h2 = markdown_heading_style(2);
        let h6 = markdown_heading_style(6);

        assert!(h1.line_height_scale > h1.font_scale);
        assert!(h2.line_height_scale > h2.font_scale);
        assert!(h6.line_height_scale > h6.font_scale);
    }

    #[test]
    fn markdown_lists_use_regular_paragraph_typography() {
        let list = markdown_line_style(&LineKind::MarkdownListItem, None)
            .expect("list items should have a style");
        let paragraph = markdown_line_style(&LineKind::MarkdownParagraph, None)
            .expect("paragraphs should have a style");

        assert_eq!(list.font_variant, paragraph.font_variant);
        assert_eq!(list.color, paragraph.color);
        assert_eq!(list.font_scale, paragraph.font_scale);
        assert_eq!(list.line_height_scale, paragraph.line_height_scale);
    }

    #[test]
    fn leaves_plain_raw_line_without_markdown_override() {
        assert!(markdown_render_override_for_raw("plain text").is_none());
    }
}
#[allow(unused_imports)]
use super::*;
