use crate::buffer::Document;
use crate::links::{extract_script_links, render_script_link_text};
use crate::model::LineKind;

use super::shared::{line_is_only_image_embed, parsed_line};

pub(super) fn parse(document: &Document) -> Vec<crate::model::ParsedLine> {
    let mut parsed = Vec::with_capacity(document.line_count());
    let mut previous_kind = LineKind::Empty;

    for (line_index, raw) in document.lines().iter().enumerate() {
        let next_raw = document.lines().get(line_index.saturating_add(1));
        let kind = classify_line(raw, &previous_kind, next_raw.map(String::as_str));
        previous_kind = kind.clone();
        parsed.push(parsed_line(raw, kind, None));
    }

    parsed
}

fn classify_line(raw: &str, previous_kind: &LineKind, next_raw: Option<&str>) -> LineKind {
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

    if next_raw.is_some_and(|next| !next.trim().is_empty()) && is_character(trimmed) {
        return LineKind::Character;
    }

    if line_is_only_image_embed(trimmed) {
        return LineKind::Action;
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
    ["INT.", "EXT.", "EST.", "INT/EXT.", "I/E."]
        .iter()
        .any(|prefix| upper.starts_with(prefix))
}

fn is_transition(line: &str) -> bool {
    let upper = line.to_uppercase();
    upper.ends_with(" TO:")
        || upper == "CUT TO:"
        || upper == "FADE OUT."
        || upper == "FADE TO BLACK."
}

fn is_character(line: &str) -> bool {
    if is_character_cue_text(line) {
        return true;
    }

    linked_character_cue_text(line)
        .map(|rendered| is_character_cue_text(&rendered.to_uppercase()))
        .unwrap_or(false)
}

fn linked_character_cue_text(line: &str) -> Option<String> {
    let links = extract_script_links(line);
    let [link] = links.as_slice() else {
        return None;
    };

    if !link.label.chars().any(|ch| ch.is_ascii_uppercase()) {
        return None;
    }

    let chars = line.chars().collect::<Vec<_>>();
    if !chars[..link.span.start].iter().all(|ch| ch.is_whitespace()) {
        return None;
    }

    let after = chars[link.span.end..].iter().collect::<String>();
    let trimmed_after = after.trim();
    if !trimmed_after.is_empty() && !is_parenthetical(trimmed_after) {
        return None;
    }

    Some(render_script_link_text(line).text)
}

fn is_character_cue_text(line: &str) -> bool {
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

        let parsed = parse(&doc);

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
        let parsed = parse(&doc);

        assert_eq!(parsed[0].kind, LineKind::SceneHeading);
        assert_eq!(parsed[1].kind, LineKind::Action);
    }

    #[test]
    fn linked_title_case_character_cue_starts_dialogue() {
        let doc = Document::from_text("[Eoghan]\nIt is just text.");
        let parsed = parse(&doc);

        assert_eq!(parsed[0].kind, LineKind::Character);
        assert_eq!(parsed[0].script_links.len(), 1);
        assert_eq!(parsed[0].script_links[0].label, "Eoghan");
        assert_eq!(parsed[0].script_links[0].target, "eoghan");
        assert_eq!(parsed[1].kind, LineKind::Dialogue);
    }

    #[test]
    fn linked_mentions_with_connective_remain_action() {
        let doc = Document::from_text("EXT. forest road.\n\n[Eoghan] and [Thorin]\nim pretty sure");
        let parsed = parse(&doc);

        assert_eq!(parsed[2].kind, LineKind::Action);
        assert_eq!(parsed[2].script_links.len(), 2);
        assert_eq!(parsed[3].kind, LineKind::Action);
    }

    #[test]
    fn linked_mention_inside_action_sentence_remains_action() {
        let doc = Document::from_text(
            "EXT. forest road.\n\n[Eoghan] and [Thorin] are dilly dallying along the forrest road\nwhen suddenly [Elizah] appears.",
        );
        let parsed = parse(&doc);

        assert_eq!(parsed[2].kind, LineKind::Action);
        assert_eq!(parsed[3].kind, LineKind::Action);
        assert_eq!(parsed[3].script_links.len(), 1);
        assert_eq!(parsed[3].script_links[0].target, "elizah");
    }

    #[test]
    fn isolated_all_caps_shot_remains_action() {
        let doc = Document::from_text(
            "EXT. FRONT PORCH - NIGHT\n\nRINGS THE DOORBELL.\n\nThe door opens.",
        );
        let parsed = parse(&doc);

        assert_eq!(parsed[2].kind, LineKind::Action);
        assert_eq!(parsed[4].kind, LineKind::Action);
    }

    #[test]
    fn all_caps_cue_with_dialogue_remains_character() {
        let doc = Document::from_text("ON WILL AND SANDRA\nWe were part of the same equation.");
        let parsed = parse(&doc);

        assert_eq!(parsed[0].kind, LineKind::Character);
        assert_eq!(parsed[1].kind, LineKind::Dialogue);
    }

    #[test]
    fn extracts_image_embeds_and_resets_dialogue_context() {
        let doc = Document::from_text("SARAH\nHello.\n![door](refs/door.png)\nThe room is quiet.");
        let parsed = parse(&doc);

        assert_eq!(parsed[2].kind, LineKind::Action);
        assert_eq!(parsed[2].image_embeds.len(), 1);
        assert_eq!(parsed[2].image_embeds[0].target, "refs/door.png");
        assert_eq!(parsed[3].kind, LineKind::Action);
    }
}
