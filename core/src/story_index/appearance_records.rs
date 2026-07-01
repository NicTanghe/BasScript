use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::{
    CanvasNodeKind, Document, DocumentFormat, LineKind, ScriptLink, extract_script_links,
    is_valid_target_key, parse_canvas_document, parse_document_with_format,
    story_index::{
        EntityIndexBuild, IndexedFileKind, IndexedWorkspaceFile, StoryIndexError,
        StoryIndexSceneRecord,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryIndexAppearanceRole {
    CharacterCue,
    DialogueSpeaker,
    ActionMention,
    DialogueMention,
    ParentheticalMention,
    SceneHeading,
    CanvasText,
    CanvasFile,
    CanvasLink,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryIndexAppearanceRecord {
    pub target: String,
    pub entity_type: Option<String>,
    pub entity_name: Option<String>,
    pub source_path: PathBuf,
    pub relative_path: String,
    pub scene_key: Option<String>,
    pub line: usize,
    pub column: usize,
    pub line_kind: String,
    pub role: StoryIndexAppearanceRole,
    pub raw_snippet: String,
}

#[derive(Clone, Debug)]
struct EntitySummary {
    entity_type: String,
    name: String,
}

pub(super) fn build_appearance_index(
    files: &[IndexedWorkspaceFile],
    scenes: &[StoryIndexSceneRecord],
    entities: &EntityIndexBuild,
) -> Result<Vec<StoryIndexAppearanceRecord>, StoryIndexError> {
    let entity_lookup = entity_summary_lookup(entities);
    let mut appearances = Vec::<StoryIndexAppearanceRecord>::new();

    for file in files
        .iter()
        .filter(|file| file.kind == IndexedFileKind::Fountain)
    {
        appearances.extend(extract_fountain_appearances(file, scenes, &entity_lookup)?);
    }

    let entity_reference_lookup = entity_reference_lookup(entities);
    for file in files
        .iter()
        .filter(|file| file.kind == IndexedFileKind::Canvas)
    {
        appearances.extend(extract_canvas_appearances(
            file,
            &entity_lookup,
            &entity_reference_lookup,
        )?);
    }

    appearances.sort_by(|left, right| {
        left.source_path
            .cmp(&right.source_path)
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.column.cmp(&right.column))
            .then_with(|| {
                left.role
                    .as_database_value()
                    .cmp(right.role.as_database_value())
            })
            .then_with(|| left.target.cmp(&right.target))
    });
    Ok(appearances)
}

fn extract_canvas_appearances(
    file: &IndexedWorkspaceFile,
    entity_lookup: &BTreeMap<String, EntitySummary>,
    entity_reference_lookup: &BTreeMap<String, String>,
) -> Result<Vec<StoryIndexAppearanceRecord>, StoryIndexError> {
    let text = fs::read_to_string(&file.path)?;
    let Ok(canvas) = parse_canvas_document(&text) else {
        return Ok(Vec::new());
    };
    let mut appearances = Vec::<StoryIndexAppearanceRecord>::new();

    for (node_index, node) in canvas.nodes.iter().enumerate() {
        match &node.kind {
            CanvasNodeKind::Text { text } => {
                for (line_index, line) in text.lines().enumerate() {
                    for link in extract_script_links(line) {
                        appearances.push(appearance_record(
                            file,
                            None,
                            line_index,
                            link.span.start,
                            "canvas_text",
                            StoryIndexAppearanceRole::CanvasText,
                            line,
                            &link.target,
                            entity_lookup,
                        ));
                    }
                }
            }
            CanvasNodeKind::File { file: reference } => {
                if let Some(target) =
                    target_for_canvas_reference(reference, entity_reference_lookup)
                {
                    appearances.push(appearance_record(
                        file,
                        None,
                        node_index,
                        0,
                        "canvas_file",
                        StoryIndexAppearanceRole::CanvasFile,
                        reference,
                        &target,
                        entity_lookup,
                    ));
                }
            }
            CanvasNodeKind::Link { url } => {
                if let Some(target) = target_for_canvas_reference(url, entity_reference_lookup) {
                    appearances.push(appearance_record(
                        file,
                        None,
                        node_index,
                        0,
                        "canvas_link",
                        StoryIndexAppearanceRole::CanvasLink,
                        url,
                        &target,
                        entity_lookup,
                    ));
                }
            }
            CanvasNodeKind::Group { .. } | CanvasNodeKind::Unknown { .. } => {}
        }
    }

    Ok(appearances)
}

fn extract_fountain_appearances(
    file: &IndexedWorkspaceFile,
    scenes: &[StoryIndexSceneRecord],
    entity_lookup: &BTreeMap<String, EntitySummary>,
) -> Result<Vec<StoryIndexAppearanceRecord>, StoryIndexError> {
    let text = fs::read_to_string(&file.path)?;
    let document = Document::from_text(&text);
    let parsed = parse_document_with_format(&document, DocumentFormat::Fountain);
    let file_scenes = scenes
        .iter()
        .filter(|scene| scene.source_path == file.path)
        .collect::<Vec<_>>();
    let mut appearances = Vec::<StoryIndexAppearanceRecord>::new();
    let mut current_speaker = None::<String>;

    for (line_index, parsed_line) in parsed.iter().enumerate() {
        let scene_key = scene_key_for_line(&file_scenes, line_index);
        let raw_snippet = parsed_line.raw.clone();

        for link in &parsed_line.script_links {
            let role = role_for_link_line_kind(&parsed_line.kind);
            appearances.push(appearance_record(
                file,
                scene_key.clone(),
                line_index,
                link.span.start,
                line_kind_label(&parsed_line.kind),
                role,
                &raw_snippet,
                &link.target,
                entity_lookup,
            ));
        }

        match parsed_line.kind {
            LineKind::Character => {
                current_speaker = linked_character_speaker(&parsed_line.script_links);
            }
            LineKind::Dialogue => {
                if let Some(target) = current_speaker.as_deref() {
                    appearances.push(appearance_record(
                        file,
                        scene_key,
                        line_index,
                        0,
                        line_kind_label(&parsed_line.kind),
                        StoryIndexAppearanceRole::DialogueSpeaker,
                        &raw_snippet,
                        target,
                        entity_lookup,
                    ));
                }
            }
            LineKind::Parenthetical => {}
            LineKind::Empty
            | LineKind::SceneHeading
            | LineKind::Action
            | LineKind::Transition
            | LineKind::MarkdownHeading
            | LineKind::MarkdownListItem
            | LineKind::MarkdownQuote
            | LineKind::MarkdownCodeFence
            | LineKind::MarkdownCode
            | LineKind::MarkdownRule
            | LineKind::MarkdownParagraph => {
                current_speaker = None;
            }
        }
    }

    Ok(appearances)
}

fn linked_character_speaker(links: &[ScriptLink]) -> Option<String> {
    let [link] = links else {
        return None;
    };
    Some(link.target.clone())
}

fn scene_key_for_line(scenes: &[&StoryIndexSceneRecord], line: usize) -> Option<String> {
    scenes
        .iter()
        .find(|scene| scene.start_line <= line && scene.end_line >= line)
        .map(|scene| scene.scene_key.clone())
}

fn role_for_link_line_kind(kind: &LineKind) -> StoryIndexAppearanceRole {
    match kind {
        LineKind::Character => StoryIndexAppearanceRole::CharacterCue,
        LineKind::Dialogue => StoryIndexAppearanceRole::DialogueMention,
        LineKind::Parenthetical => StoryIndexAppearanceRole::ParentheticalMention,
        LineKind::SceneHeading => StoryIndexAppearanceRole::SceneHeading,
        LineKind::Empty
        | LineKind::Action
        | LineKind::Transition
        | LineKind::MarkdownHeading
        | LineKind::MarkdownListItem
        | LineKind::MarkdownQuote
        | LineKind::MarkdownCodeFence
        | LineKind::MarkdownCode
        | LineKind::MarkdownRule
        | LineKind::MarkdownParagraph => StoryIndexAppearanceRole::ActionMention,
    }
}

fn appearance_record(
    file: &IndexedWorkspaceFile,
    scene_key: Option<String>,
    line: usize,
    column: usize,
    line_kind: &'static str,
    role: StoryIndexAppearanceRole,
    raw_snippet: &str,
    target: &str,
    entity_lookup: &BTreeMap<String, EntitySummary>,
) -> StoryIndexAppearanceRecord {
    let summary = entity_lookup.get(target);
    StoryIndexAppearanceRecord {
        target: target.to_string(),
        entity_type: summary.map(|summary| summary.entity_type.clone()),
        entity_name: summary.map(|summary| summary.name.clone()),
        source_path: file.path.clone(),
        relative_path: file.relative_display_path(),
        scene_key,
        line,
        column,
        line_kind: line_kind.to_string(),
        role,
        raw_snippet: raw_snippet.to_string(),
    }
}

fn entity_summary_lookup(entities: &EntityIndexBuild) -> BTreeMap<String, EntitySummary> {
    entities
        .entities
        .iter()
        .map(|record| {
            (
                record.document.metadata.target.clone(),
                EntitySummary {
                    entity_type: record.document.metadata.entity_type.clone(),
                    name: record.document.metadata.name.clone(),
                },
            )
        })
        .collect()
}

fn entity_reference_lookup(entities: &EntityIndexBuild) -> BTreeMap<String, String> {
    let mut lookup = BTreeMap::<String, String>::new();

    for entity in &entities.entities {
        let target = entity.document.metadata.target.clone();
        insert_reference_key(&mut lookup, &target, &target);
        insert_reference_key(&mut lookup, &entity.relative_path, &target);

        let entity_path = PathBuf::from(&entity.relative_path);
        if let Some(file_name) = entity_path.file_name() {
            insert_reference_key(&mut lookup, &file_name.to_string_lossy(), &target);
        }
        if let Some(file_stem) = entity_path.file_stem() {
            insert_reference_key(&mut lookup, &file_stem.to_string_lossy(), &target);
        }
        if let Some(extension) = entity_path
            .extension()
            .and_then(|extension| extension.to_str())
        {
            let extension = format!(".{extension}");
            if entity.relative_path.ends_with(&extension) {
                let without_extension =
                    &entity.relative_path[..entity.relative_path.len() - extension.len()];
                insert_reference_key(&mut lookup, without_extension, &target);
            }
        }
    }

    lookup
}

fn insert_reference_key(lookup: &mut BTreeMap<String, String>, reference: &str, target: &str) {
    let key = normalized_canvas_reference(reference);
    if !key.is_empty() {
        lookup.insert(key, target.to_string());
    }
}

fn target_for_canvas_reference(
    reference: &str,
    entity_reference_lookup: &BTreeMap<String, String>,
) -> Option<String> {
    if reference.starts_with("http://")
        || reference.starts_with("https://")
        || reference.starts_with("mailto:")
    {
        return None;
    }

    let normalized = normalized_canvas_reference(reference);
    if normalized.is_empty() {
        return None;
    }
    if is_valid_target_key(&normalized) && entity_reference_lookup.contains_key(&normalized) {
        return Some(normalized);
    }

    entity_reference_lookup.get(&normalized).cloned()
}

fn normalized_canvas_reference(reference: &str) -> String {
    reference
        .trim()
        .trim_start_matches("./")
        .replace('\\', "/")
        .to_lowercase()
}

fn line_kind_label(kind: &LineKind) -> &'static str {
    match kind {
        LineKind::Empty => "empty",
        LineKind::SceneHeading => "scene_heading",
        LineKind::Action => "action",
        LineKind::Character => "character",
        LineKind::Dialogue => "dialogue",
        LineKind::Parenthetical => "parenthetical",
        LineKind::Transition => "transition",
        LineKind::MarkdownHeading => "markdown_heading",
        LineKind::MarkdownListItem => "markdown_list_item",
        LineKind::MarkdownQuote => "markdown_quote",
        LineKind::MarkdownCodeFence => "markdown_code_fence",
        LineKind::MarkdownCode => "markdown_code",
        LineKind::MarkdownRule => "markdown_rule",
        LineKind::MarkdownParagraph => "markdown_paragraph",
    }
}

impl IndexedWorkspaceFile {
    fn relative_display_path(&self) -> String {
        self.relative_path.clone()
    }
}

impl StoryIndexAppearanceRole {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::CharacterCue => "character_cue",
            Self::DialogueSpeaker => "dialogue_speaker",
            Self::ActionMention => "action_mention",
            Self::DialogueMention => "dialogue_mention",
            Self::ParentheticalMention => "parenthetical_mention",
            Self::SceneHeading => "scene_heading",
            Self::CanvasText => "canvas_text",
            Self::CanvasFile => "canvas_file",
            Self::CanvasLink => "canvas_link",
        }
    }

    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "character_cue" => Some(Self::CharacterCue),
            "dialogue_speaker" => Some(Self::DialogueSpeaker),
            "action_mention" => Some(Self::ActionMention),
            "dialogue_mention" => Some(Self::DialogueMention),
            "parenthetical_mention" => Some(Self::ParentheticalMention),
            "scene_heading" => Some(Self::SceneHeading),
            "canvas_text" => Some(Self::CanvasText),
            "canvas_file" => Some(Self::CanvasFile),
            "canvas_link" => Some(Self::CanvasLink),
            _ => None,
        }
    }
}
