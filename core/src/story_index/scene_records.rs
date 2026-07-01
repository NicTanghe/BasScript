use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    Document, DocumentFormat, LineKind, parse_document_with_format,
    story_index::{IndexedFileKind, IndexedWorkspaceFile, StoryIndexError},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryIndexSceneRecord {
    pub scene_key: String,
    pub source_path: PathBuf,
    pub relative_path: String,
    pub scene_ordinal: usize,
    pub heading_text: String,
    pub normalized_heading: String,
    pub start_line: usize,
    pub end_line: usize,
    pub script_order: usize,
    pub location_text: Option<String>,
    pub time_of_day_text: Option<String>,
}

pub(super) fn build_scene_index(
    files: &[IndexedWorkspaceFile],
) -> Result<Vec<StoryIndexSceneRecord>, StoryIndexError> {
    let mut scenes = Vec::<StoryIndexSceneRecord>::new();
    let mut script_order = 0usize;

    for file in files
        .iter()
        .filter(|file| file.kind == IndexedFileKind::Fountain)
    {
        let mut file_scenes = extract_scenes_from_file(file, script_order)?;
        script_order = script_order.saturating_add(file_scenes.len());
        scenes.append(&mut file_scenes);
    }

    Ok(scenes)
}

fn extract_scenes_from_file(
    file: &IndexedWorkspaceFile,
    script_order_offset: usize,
) -> Result<Vec<StoryIndexSceneRecord>, StoryIndexError> {
    let text = fs::read_to_string(&file.path)?;
    let document = Document::from_text(&text);
    let parsed = parse_document_with_format(&document, DocumentFormat::Fountain);
    let scene_starts = parsed
        .iter()
        .enumerate()
        .filter_map(|(line, parsed)| (parsed.kind == LineKind::SceneHeading).then_some(line))
        .collect::<Vec<_>>();

    let mut scenes = Vec::<StoryIndexSceneRecord>::with_capacity(scene_starts.len());
    for (index, start_line) in scene_starts.iter().copied().enumerate() {
        let end_line = scene_starts
            .get(index + 1)
            .map(|next_start| next_start.saturating_sub(1))
            .unwrap_or_else(|| document.line_count().saturating_sub(1))
            .max(start_line);
        let heading_text = document
            .line(start_line)
            .unwrap_or_default()
            .trim()
            .to_string();
        let scene_ordinal = index + 1;
        let (location_text, time_of_day_text) = infer_location_and_time(&heading_text);

        scenes.push(StoryIndexSceneRecord {
            scene_key: scene_key(&file.path, scene_ordinal),
            source_path: file.path.clone(),
            relative_path: file.relative_path.clone(),
            scene_ordinal,
            heading_text: heading_text.clone(),
            normalized_heading: normalize_scene_heading(&heading_text),
            start_line,
            end_line,
            script_order: script_order_offset + index,
            location_text,
            time_of_day_text,
        });
    }

    Ok(scenes)
}

fn scene_key(path: &Path, scene_ordinal: usize) -> String {
    format!("{}#scene-{scene_ordinal:04}", path.to_string_lossy())
}

fn normalize_scene_heading(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn infer_location_and_time(heading: &str) -> (Option<String>, Option<String>) {
    let body = strip_scene_prefix(heading.trim());
    if body.is_empty() {
        return (None, None);
    }

    let parts = body
        .split(" - ")
        .map(|part| part.trim().trim_matches('.').trim())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    match parts.as_slice() {
        [] => (None, None),
        [location] => (Some((*location).to_string()), None),
        [location, rest @ ..] => (
            Some((*location).to_string()),
            rest.last().map(|time| (*time).to_string()),
        ),
    }
}

fn strip_scene_prefix(heading: &str) -> &str {
    let trimmed = heading.trim_start();
    let upper = trimmed.to_ascii_uppercase();
    for prefix in ["INT/EXT.", "INT./EXT.", "I/E.", "INT.", "EXT.", "EST."] {
        if upper.starts_with(prefix) {
            return trimmed[prefix.len()..].trim();
        }
    }
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_location_and_time_from_standard_heading() {
        let (location, time) = infer_location_and_time("INT. KITCHEN - NIGHT");

        assert_eq!(location.as_deref(), Some("KITCHEN"));
        assert_eq!(time.as_deref(), Some("NIGHT"));
    }

    #[test]
    fn infers_location_without_time() {
        let (location, time) = infer_location_and_time("Ext. forest road.");

        assert_eq!(location.as_deref(), Some("forest road"));
        assert_eq!(time, None);
    }
}
