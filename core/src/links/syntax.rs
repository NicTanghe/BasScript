use std::ops::RangeInclusive;

use super::{LinkDisplayText, ScriptLink, ScriptLinkSyntax};

pub fn extract_script_links(input: &str) -> Vec<ScriptLink> {
    let chars = input.chars().collect::<Vec<_>>();
    let mut links = Vec::<ScriptLink>::new();
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] != '[' {
            index += 1;
            continue;
        }
        if index > 0 && chars[index - 1] == '!' {
            index += 1;
            continue;
        }

        let Some(label_end) = chars[index + 1..]
            .iter()
            .position(|ch| *ch == ']')
            .map(|offset| index + 1 + offset)
        else {
            break;
        };

        let label = chars[index + 1..label_end].iter().collect::<String>();
        if label.is_empty() {
            index += 1;
            continue;
        }

        if chars.get(label_end + 1) == Some(&'(') {
            let Some(target_end) = chars[label_end + 2..]
                .iter()
                .position(|ch| *ch == ')')
                .map(|offset| label_end + 2 + offset)
            else {
                index += 1;
                continue;
            };
            let target = chars[label_end + 2..target_end].iter().collect::<String>();
            if is_valid_target_key(&target) {
                links.push(ScriptLink {
                    span: index..target_end + 1,
                    label,
                    target,
                    syntax: ScriptLinkSyntax::LabelledTarget,
                });
                index = target_end + 1;
                continue;
            }

            index += 1;
            continue;
        }

        if let Some(target) = target_key_from_bare_label(&label) {
            links.push(ScriptLink {
                span: index..label_end + 1,
                label: label.clone(),
                target,
                syntax: ScriptLinkSyntax::TargetOnly,
            });
            index = label_end + 1;
            continue;
        }

        index += 1;
    }

    links
}

pub fn render_script_link_text(input: &str) -> LinkDisplayText {
    let chars = input.chars().collect::<Vec<_>>();
    let links = extract_script_links(input);
    let mut rendered = String::new();
    let mut display_to_raw = vec![0usize];
    let mut cursor = 0usize;

    for link in &links {
        while cursor < link.span.start {
            rendered.push(chars[cursor]);
            display_to_raw.push(cursor + 1);
            cursor += 1;
        }

        let label_raw_start = link.span.start + 1;
        for (offset, ch) in link.label.chars().enumerate() {
            rendered.push(ch);
            display_to_raw.push(label_raw_start + offset + 1);
        }

        if let Some(last) = display_to_raw.last_mut() {
            *last = link.span.end;
        }

        cursor = link.span.end;
    }

    while cursor < chars.len() {
        rendered.push(chars[cursor]);
        display_to_raw.push(cursor + 1);
        cursor += 1;
    }

    LinkDisplayText {
        text: rendered,
        display_to_raw,
    }
}

pub fn is_valid_target_key(target: &str) -> bool {
    if target.is_empty() {
        return false;
    }

    let parts = target.split('-').collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return false;
    }

    parts.iter().all(|part| {
        part.chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
    })
}

fn target_key_from_bare_label(label: &str) -> Option<String> {
    if is_valid_target_key(label) {
        return Some(label.to_owned());
    }

    if !looks_like_title_case_mention(label) {
        return None;
    }

    let target = slugify_bare_label(label)?;
    is_valid_target_key(&target).then_some(target)
}

fn looks_like_title_case_mention(label: &str) -> bool {
    let trimmed = label.trim();
    if trimmed.is_empty() || trimmed != label {
        return false;
    }

    let has_upper = trimmed.chars().any(|ch| ch.is_ascii_uppercase());
    let has_lower = trimmed.chars().any(|ch| ch.is_ascii_lowercase());
    if !(has_upper && has_lower) {
        return false;
    }

    trimmed.split_whitespace().all(|word| {
        word.chars()
            .find(|ch| ch.is_ascii_alphabetic())
            .is_some_and(|ch| ch.is_ascii_uppercase())
    })
}

fn slugify_bare_label(label: &str) -> Option<String> {
    let mut target = String::new();
    let mut previous_was_separator = true;

    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            target.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
            continue;
        }

        if matches!(ch, ' ' | '\t' | '-' | '_' | '\'' | '.') {
            if !previous_was_separator {
                target.push('-');
                previous_was_separator = true;
            }
            continue;
        }

        return None;
    }

    while target.ends_with('-') {
        target.pop();
    }

    (!target.is_empty()).then_some(target)
}

pub fn script_link_visible_column_range(link: &ScriptLink) -> RangeInclusive<usize> {
    let start = link.span.start.saturating_add(1);
    let end = match link.syntax {
        ScriptLinkSyntax::TargetOnly => link.span.end.saturating_sub(1),
        ScriptLinkSyntax::LabelledTarget => start.saturating_add(link.label.chars().count()),
    };
    start..=end
}

pub fn script_link_contains_visible_column(link: &ScriptLink, column: usize) -> bool {
    script_link_visible_column_range(link).contains(&column)
}
