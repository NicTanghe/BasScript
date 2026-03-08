fn markdown_visual_text(parsed_line: &ParsedLine) -> Option<(usize, String, Option<bool>)> {
    match parsed_line.kind {
        LineKind::MarkdownHeading => {
            let (consumed, rendered) = markdown_heading_visual(&parsed_line.raw);
            Some((consumed, rendered, None))
        }
        LineKind::MarkdownListItem => Some(markdown_list_item_visual(&parsed_line.raw)),
        LineKind::MarkdownQuote => {
            let (consumed, rendered) = markdown_quote_visual(&parsed_line.raw);
            Some((consumed, rendered, None))
        }
        LineKind::MarkdownRule => Some((0, "────────────────────────".to_string(), None)),
        LineKind::MarkdownCodeFence => Some((0, "```".to_string(), None)),
        _ => None,
    }
}

fn markdown_line_style(parsed_line: &ParsedLine) -> Option<LineRenderStyle> {
    match parsed_line.kind {
        LineKind::MarkdownHeading => Some(markdown_heading_style(
            parsed_line.markdown_heading_level.unwrap_or(1),
        )),
        LineKind::MarkdownListItem => Some(LineRenderStyle::new(
            FontVariant::Regular,
            COLOR_MARKDOWN_LIST,
            1.0,
            1.0,
        )),
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

fn markdown_heading_style(level: u8) -> LineRenderStyle {
    let font_scale = match level.clamp(1, 6) {
        1 => 1.80,
        2 => 1.55,
        3 => 1.35,
        4 => 1.20,
        5 => 1.05,
        _ => 0.95,
    };

    LineRenderStyle::new(
        FontVariant::Bold,
        COLOR_MARKDOWN_HEADING,
        font_scale,
        font_scale.max(1.0),
    )
}

fn markdown_heading_visual(raw: &str) -> (usize, String) {
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

fn markdown_quote_visual(raw: &str) -> (usize, String) {
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

fn markdown_list_item_visual(raw: &str) -> (usize, String, Option<bool>) {
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
        return (leading.saturating_add(marker_end), format!("• {text}"), None);
    }

    if let Some((prefix, content_start)) = ordered_list_content_start(&trimmed) {
        if let Some((consumed, checked, checklist_content_start)) =
            markdown_checklist_marker(&trimmed, content_start)
        {
            let text = trimmed[checklist_content_start..].iter().collect::<String>();
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

fn markdown_checklist_marker(chars: &[char], start: usize) -> Option<(usize, bool, usize)> {
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

fn leading_markdown_whitespace(raw: &str) -> usize {
    raw.chars()
        .take_while(|ch| matches!(*ch, ' ' | '\t'))
        .count()
}

fn unordered_list_content_start(chars: &[char]) -> Option<usize> {
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

fn ordered_list_content_start(chars: &[char]) -> Option<(String, usize)> {
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
