use super::*;
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path =
            env::temp_dir().join(format!("basscript-links-{}-{}", std::process::id(), unique));
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }
        fs::write(path, contents).expect("write file");
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn kitchen_door_markdown() -> &'static str {
    "---\nid: obj_door_kitchen_main_001\ntarget: door-kitchen-main\ntype: prop\nname: Kitchen main door\naliases:\n  - kitchen door\n  - main kitchen door\nstatus: draft\n---\nCanonical notes.\n"
}

#[test]
fn extracts_direct_and_labelled_links() {
    let links = extract_script_links(
        "He opens [door-kitchen-main] and points at [that door](door-kitchen-main).",
    );

    assert_eq!(links.len(), 2);
    assert_eq!(links[0].label, "door-kitchen-main");
    assert_eq!(links[0].target, "door-kitchen-main");
    assert_eq!(links[0].syntax, ScriptLinkSyntax::TargetOnly);
    assert_eq!(links[1].label, "that door");
    assert_eq!(links[1].target, "door-kitchen-main");
    assert_eq!(links[1].syntax, ScriptLinkSyntax::LabelledTarget);
}

#[test]
fn ignores_non_slug_targets() {
    let links = extract_script_links("Ignore [site](https://example.com) and [not a target].");
    assert!(links.is_empty());
}

#[test]
fn renders_labelled_links_as_visible_text() {
    let rendered = render_script_link_text("Open [that door](door-kitchen-main) now.");

    assert_eq!(rendered.text, "Open that door now.");
    assert_eq!(rendered.display_to_raw.last().copied(), Some(40));
}

#[test]
fn computes_visible_click_range_for_labelled_links() {
    let link = extract_script_links("[that door](door-kitchen-main)")
        .into_iter()
        .next()
        .unwrap();

    assert!(script_link_contains_visible_column(&link, 1));
    assert!(script_link_contains_visible_column(&link, 9));
    assert!(script_link_contains_visible_column(&link, 10));
    assert!(!script_link_contains_visible_column(&link, 11));
}

#[test]
fn loads_entity_document_from_yaml_front_matter() {
    let document =
        EntityDocument::from_markdown("door-kitchen-main.md", kitchen_door_markdown()).unwrap();

    assert_eq!(document.metadata.id, "obj_door_kitchen_main_001");
    assert_eq!(document.metadata.target, "door-kitchen-main");
    assert_eq!(document.metadata.entity_type, "prop");
    assert_eq!(document.metadata.name, "Kitchen main door");
    assert_eq!(
        document.metadata.aliases,
        vec!["kitchen door".to_owned(), "main kitchen door".to_owned()]
    );
    assert_eq!(document.metadata.status.as_deref(), Some("draft"));
}

#[test]
fn ignores_unknown_yaml_blocks_after_known_front_matter_fields() {
    let markdown = r#"---
id: place_cinder_ward_001
target: cinder-ward
type: place
name: Cinder Ward
aliases:
  - the Ward
status: energized
tags:
  - undercity
  - industrial
controlled_by: gilded-conclave
home_to:
  - cinder-union
description: >
  A flood-prone worker district.
  It keeps the city functioning.
---
Notes.
"#;
    let document = EntityDocument::from_markdown("cinder-ward.md", markdown).unwrap();

    assert_eq!(document.metadata.target, "cinder-ward");
    assert_eq!(document.metadata.entity_type, "place");
    assert_eq!(document.metadata.name, "Cinder Ward");
    assert_eq!(document.metadata.aliases, vec!["the Ward".to_owned()]);
    assert_eq!(document.metadata.status.as_deref(), Some("energized"));
}

#[test]
fn rejects_target_filename_mismatch() {
    let error = EntityDocument::from_markdown("other.md", kitchen_door_markdown()).unwrap_err();
    assert!(matches!(error, LinkError::FilenameTargetMismatch { .. }));
}

#[test]
fn resolves_explicit_links_by_target_and_does_not_promote_local_labels() {
    let root = TestDir::new();
    root.write("door-kitchen-main.md", kitchen_door_markdown());
    let catalog = EntityCatalog::load_from_dir(root.path()).unwrap();
    let link = extract_script_links("[that door](door-kitchen-main)")
        .into_iter()
        .next()
        .unwrap();

    let resolved = catalog.resolve_script_link(&link, root.path());
    match resolved {
        MentionResolution::Resolved(entity) => {
            assert_eq!(entity.source, ResolutionSource::ExplicitLink);
            assert_eq!(entity.target, "door-kitchen-main");
        }
        other => panic!("expected explicit resolution, got {other:?}"),
    }

    let alias_lookup = catalog.resolve_mention("that door", None, root.path());
    assert!(!matches!(alias_lookup, MentionResolution::Resolved(_)));
}

#[test]
fn resolves_confirmed_alias_before_suggestions() {
    let root = TestDir::new();
    root.write("door-kitchen-main.md", kitchen_door_markdown());
    let catalog = EntityCatalog::load_from_dir(root.path()).unwrap();

    let resolved = catalog.resolve_mention("kitchen door", None, root.path());
    match resolved {
        MentionResolution::Resolved(entity) => {
            assert_eq!(entity.source, ResolutionSource::Alias);
            assert_eq!(entity.target, "door-kitchen-main");
        }
        other => panic!("expected alias resolution, got {other:?}"),
    }
}

#[test]
fn returns_entity_matching_suggestions_after_alias_lookup() {
    let root = TestDir::new();
    root.write("door-kitchen-main.md", kitchen_door_markdown());
    let catalog = EntityCatalog::load_from_dir(root.path()).unwrap();

    let resolved = catalog.resolve_mention("main door", None, root.path());
    match resolved {
        MentionResolution::Suggested(suggested) => {
            assert_eq!(suggested.suggestions[0].target, "door-kitchen-main");
            assert_eq!(
                suggested.suggestions[0].origin,
                SuggestionOrigin::EntityMatch
            );
        }
        other => panic!("expected suggestion resolution, got {other:?}"),
    }
}

#[test]
fn returns_unresolved_with_scaffold_for_missing_explicit_target() {
    let root = TestDir::new();
    let catalog = EntityCatalog::default();

    let resolved = catalog.resolve_mention("that door", Some("door-kitchen-main"), root.path());
    match resolved {
        MentionResolution::Unresolved(unresolved) => {
            assert_eq!(unresolved.reason, UnresolvedReason::MissingTargetFile);
            let scaffold = unresolved.scaffold.expect("expected scaffold");
            assert!(scaffold.path.ends_with("door-kitchen-main.md"));
            assert!(scaffold.markdown.contains("target: door-kitchen-main"));
        }
        other => panic!("expected unresolved resolution, got {other:?}"),
    }
}
