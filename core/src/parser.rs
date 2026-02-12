use crate::buffer::Document;
use crate::model::{LineKind, ParsedLine};

pub fn parse_document(document: &Document) -> Vec<ParsedLine> {
    let mut parsed = Vec::with_capacity(document.line_count());
    let mut previous_kind = LineKind::Empty;

    for raw in document.lines() {
        let kind = classify_line(raw, &previous_kind);
        previous_kind = kind.clone();

        parsed.push(ParsedLine {
            kind,
            raw: raw.clone(),
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
}
