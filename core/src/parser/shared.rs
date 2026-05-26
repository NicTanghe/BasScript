use crate::links::extract_script_links;
use crate::model::{ImageEmbed, LineKind, ParsedLine};

pub(super) fn parsed_line(
    raw: &str,
    kind: LineKind,
    markdown_heading_level: Option<u8>,
) -> ParsedLine {
    parsed_line_with_image_embeds(raw, kind, markdown_heading_level, true)
}

pub(super) fn parsed_line_with_image_embeds(
    raw: &str,
    kind: LineKind,
    markdown_heading_level: Option<u8>,
    include_image_embeds: bool,
) -> ParsedLine {
    ParsedLine {
        kind,
        raw: raw.to_owned(),
        script_links: extract_script_links(raw),
        image_embeds: include_image_embeds
            .then(|| extract_image_embeds(raw))
            .unwrap_or_default(),
        markdown_heading_level,
    }
}

pub(super) fn line_is_only_image_embed(raw: &str) -> bool {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return false;
    }

    let embeds = extract_image_embeds(trimmed);
    embeds.len() == 1
        && embeds[0].raw_start_column == 0
        && embeds[0].raw_end_column == trimmed.chars().count()
}

pub(super) fn extract_image_embeds(input: &str) -> Vec<ImageEmbed> {
    let chars = input.chars().collect::<Vec<_>>();
    let mut embeds = Vec::<ImageEmbed>::new();
    let mut index = 0usize;

    while index + 1 < chars.len() {
        if chars[index] != '!' || chars.get(index + 1) != Some(&'[') {
            index += 1;
            continue;
        }

        let Some(label_end) = chars[index + 2..]
            .iter()
            .position(|ch| *ch == ']')
            .map(|offset| index + 2 + offset)
        else {
            break;
        };

        if chars.get(label_end + 1) != Some(&'(') {
            index += 1;
            continue;
        }

        let Some(target_end) = chars[label_end + 2..]
            .iter()
            .position(|ch| *ch == ')')
            .map(|offset| label_end + 2 + offset)
        else {
            index += 1;
            continue;
        };

        let alt = chars[index + 2..label_end].iter().collect::<String>();
        let inner = chars[label_end + 2..target_end].iter().collect::<String>();
        let Some((target, title)) = parse_image_target_and_title(&inner) else {
            index += 1;
            continue;
        };

        embeds.push(ImageEmbed {
            raw_start_column: index,
            raw_end_column: target_end + 1,
            alt,
            target,
            title,
        });
        index = target_end + 1;
    }

    embeds
}

fn parse_image_target_and_title(input: &str) -> Option<(String, Option<String>)> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let chars = trimmed.chars().collect::<Vec<_>>();
    if chars.first() == Some(&'<') {
        let end = chars.iter().position(|ch| *ch == '>')?;
        if end <= 1 {
            return None;
        }
        let target = chars[1..end].iter().collect::<String>();
        let trailing = chars[end + 1..].iter().collect::<String>();
        let title = parse_image_title(trailing.trim());
        return Some((target, title));
    }

    if matches!(chars.last(), Some('"') | Some('\'')) {
        let quote = *chars.last()?;
        if let Some(open_quote) = chars[..chars.len().saturating_sub(1)]
            .iter()
            .rposition(|ch| *ch == quote)
        {
            let target_part = chars[..open_quote].iter().collect::<String>();
            if target_part.chars().last().is_some_and(char::is_whitespace) {
                let target = target_part.trim().to_owned();
                if !target.is_empty() {
                    let title = chars[open_quote + 1..chars.len().saturating_sub(1)]
                        .iter()
                        .collect::<String>();
                    return Some((target, Some(title)));
                }
            }
        }
    }

    Some((trimmed.to_owned(), None))
}

fn parse_image_title(input: &str) -> Option<String> {
    if input.len() < 2 {
        return None;
    }

    let mut chars = input.chars();
    let quote = chars.next()?;
    if !matches!(quote, '"' | '\'') {
        return None;
    }
    if !input.ends_with(quote) {
        return None;
    }

    Some(input[1..input.len().saturating_sub(1)].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_internal_script_links_per_line() {
        let parsed = parsed_line(
            "He opens [door-kitchen-main] and [that door](door-kitchen-main).",
            LineKind::Action,
            None,
        );

        assert_eq!(parsed.script_links.len(), 2);
        assert_eq!(parsed.script_links[0].target, "door-kitchen-main");
        assert_eq!(parsed.script_links[1].label, "that door");
    }

    #[test]
    fn extracts_markdown_image_embeds() {
        let parsed = parsed_line(
            "Look at ![Door](refs/door.png \"Main door\") now.",
            LineKind::Action,
            None,
        );

        assert_eq!(parsed.image_embeds.len(), 1);
        assert_eq!(parsed.image_embeds[0].raw_start_column, 8);
        assert_eq!(parsed.image_embeds[0].raw_end_column, 42);
        assert_eq!(parsed.image_embeds[0].alt, "Door");
        assert_eq!(parsed.image_embeds[0].target, "refs/door.png");
        assert_eq!(parsed.image_embeds[0].title.as_deref(), Some("Main door"));
    }

    #[test]
    fn image_embeds_can_be_disabled_for_code_lines() {
        let parsed = parsed_line_with_image_embeds(
            "![Door](refs/door.png)",
            LineKind::MarkdownCode,
            None,
            false,
        );

        assert!(parsed.image_embeds.is_empty());
    }
}
