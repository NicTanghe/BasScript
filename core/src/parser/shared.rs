use crate::links::extract_script_links;
use crate::model::{LineKind, ParsedLine};

pub(super) fn parsed_line(
    raw: &str,
    kind: LineKind,
    markdown_heading_level: Option<u8>,
) -> ParsedLine {
    ParsedLine {
        kind,
        raw: raw.to_owned(),
        script_links: extract_script_links(raw),
        markdown_heading_level,
    }
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
}
