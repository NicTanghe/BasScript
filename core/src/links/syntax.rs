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

        if is_valid_target_key(&label) {
            links.push(ScriptLink {
                span: index..label_end + 1,
                label: label.clone(),
                target: label,
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
