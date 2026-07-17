use super::{LinkDisplayText, MarkdownLink};

pub fn extract_markdown_links(input: &str) -> Vec<MarkdownLink> {
    let chars = input.chars().collect::<Vec<_>>();
    let code_mask = markdown_code_span_mask(&chars);
    let mut links = Vec::<MarkdownLink>::new();
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] != '[' || code_mask[index] || is_escaped(&chars, index) {
            index += 1;
            continue;
        }
        if index > 0 && chars[index - 1] == '!' && !is_escaped(&chars, index - 1) {
            index += 1;
            continue;
        }

        let Some(label_end) = find_label_end(&chars, index + 1) else {
            break;
        };
        let Some(target_open) = label_end
            .checked_add(1)
            .filter(|next| chars.get(*next) == Some(&'('))
        else {
            index += 1;
            continue;
        };
        let Some(target_close) = find_target_close(&chars, target_open + 1) else {
            index += 1;
            continue;
        };
        let Some(target) = markdown_link_target(&chars[target_open + 1..target_close]) else {
            index += 1;
            continue;
        };

        links.push(MarkdownLink {
            span: index..target_close + 1,
            label_span: index + 1..label_end,
            label: markdown_unescape(&chars[index + 1..label_end]),
            target,
        });
        index = target_close + 1;
    }

    links
}

pub fn render_markdown_link_text(input: &str) -> LinkDisplayText {
    let chars = input.chars().collect::<Vec<_>>();
    let links = extract_markdown_links(input);
    let code_mask = markdown_code_span_mask(&chars);
    let mut text = String::new();
    let mut display_to_raw = vec![0usize];
    let mut cursor = 0usize;

    for link in &links {
        push_unescaped_range(
            &chars,
            &code_mask,
            cursor,
            link.span.start,
            &mut text,
            &mut display_to_raw,
        );
        push_unescaped_range(
            &chars,
            &code_mask,
            link.label_span.start,
            link.label_span.end,
            &mut text,
            &mut display_to_raw,
        );
        if let Some(last) = display_to_raw.last_mut() {
            *last = link.span.end;
        }
        cursor = link.span.end;
    }

    push_unescaped_range(
        &chars,
        &code_mask,
        cursor,
        chars.len(),
        &mut text,
        &mut display_to_raw,
    );

    LinkDisplayText {
        text,
        display_to_raw,
    }
}

fn find_label_end(chars: &[char], start: usize) -> Option<usize> {
    let mut nested = 0usize;
    let mut index = start;
    while index < chars.len() {
        if is_escaped(chars, index) {
            index += 1;
            continue;
        }
        match chars[index] {
            '[' => nested += 1,
            ']' if nested == 0 => return Some(index),
            ']' => nested = nested.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }
    None
}

fn find_target_close(chars: &[char], start: usize) -> Option<usize> {
    let mut nested = 0usize;
    let mut quote = None::<char>;
    let mut index = start;

    while index < chars.len() {
        if is_escaped(chars, index) {
            index += 1;
            continue;
        }

        let ch = chars[index];
        if let Some(open_quote) = quote {
            if ch == open_quote {
                quote = None;
            }
            index += 1;
            continue;
        }
        if matches!(ch, '"' | '\'')
            && chars[..index]
                .last()
                .is_some_and(|previous| previous.is_whitespace())
        {
            quote = Some(ch);
            index += 1;
            continue;
        }

        match ch {
            '(' => nested += 1,
            ')' if nested == 0 => return Some(index),
            ')' => nested = nested.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }
    None
}

fn markdown_link_target(chars: &[char]) -> Option<String> {
    let start = chars.iter().position(|ch| !ch.is_whitespace())?;
    let end = chars.iter().rposition(|ch| !ch.is_whitespace())? + 1;
    let trimmed = &chars[start..end];

    if trimmed.first() == Some(&'<') {
        let close = trimmed
            .iter()
            .enumerate()
            .skip(1)
            .find(|(index, ch)| **ch == '>' && !is_escaped(trimmed, *index))
            .map(|(index, _)| index)?;
        return (close > 1).then(|| markdown_unescape(&trimmed[1..close]));
    }

    let target_end = trimmed
        .iter()
        .position(|ch| ch.is_whitespace())
        .unwrap_or(trimmed.len());
    (target_end > 0).then(|| markdown_unescape(&trimmed[..target_end]))
}

fn markdown_unescape(chars: &[char]) -> String {
    let mut text = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == '\\'
            && chars
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_punctuation())
        {
            text.push(chars[index + 1]);
            index += 2;
        } else {
            text.push(chars[index]);
            index += 1;
        }
    }
    text
}

fn push_unescaped_range(
    chars: &[char],
    code_mask: &[bool],
    start: usize,
    end: usize,
    text: &mut String,
    display_to_raw: &mut Vec<usize>,
) {
    let mut index = start.min(chars.len());
    let end = end.min(chars.len());
    while index < end {
        if chars[index] == '\\'
            && !code_mask.get(index).copied().unwrap_or(false)
            && chars
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_punctuation())
            && index + 1 < end
        {
            text.push(chars[index + 1]);
            display_to_raw.push(index + 2);
            index += 2;
        } else {
            text.push(chars[index]);
            display_to_raw.push(index + 1);
            index += 1;
        }
    }
}

fn is_escaped(chars: &[char], index: usize) -> bool {
    let mut backslashes = 0usize;
    let mut cursor = index;
    while cursor > 0 && chars[cursor - 1] == '\\' {
        backslashes += 1;
        cursor -= 1;
    }
    backslashes % 2 == 1
}

fn markdown_code_span_mask(chars: &[char]) -> Vec<bool> {
    let mut mask = vec![false; chars.len()];
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] != '`' || is_escaped(chars, index) {
            index += 1;
            continue;
        }

        let run_len = same_char_run_len(chars, index, '`');
        let mut search = index + run_len;
        let mut closing = None;
        while search < chars.len() {
            if chars[search] == '`'
                && !is_escaped(chars, search)
                && same_char_run_len(chars, search, '`') == run_len
            {
                closing = Some(search);
                break;
            }
            search += 1;
        }

        let Some(closing) = closing else {
            index += run_len;
            continue;
        };
        mask[index..closing + run_len].fill(true);
        index = closing + run_len;
    }

    mask
}

fn same_char_run_len(chars: &[char], start: usize, marker: char) -> usize {
    chars[start..]
        .iter()
        .take_while(|ch| **ch == marker)
        .count()
}
