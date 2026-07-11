use basscript_core::{
    EntityDocument, ScriptLinkSyntax, is_valid_target_key, scaffold_entity,
    script_link_contains_visible_column, script_link_visible_column_range,
};

impl EditorState {
    pub(crate) fn clear_script_link_target_cache(&mut self) {
        self.script_link_target_types.clear();
        self.missing_script_link_targets.clear();
    }

    pub(crate) fn ensure_current_script_link_targets_cached(&mut self) {
        let links = self
            .parsed
            .iter()
            .flat_map(|line| line.script_links.iter().cloned())
            .collect::<Vec<_>>();
        let targets = links
            .iter()
            .map(|link| link.target.clone())
            .collect::<BTreeSet<_>>();

        self.script_link_target_types
            .retain(|target, _| targets.contains(target));
        self.missing_script_link_targets
            .retain(|target| targets.contains(target));

        for link in links {
            let target = link.target.clone();
            if self.script_link_target_types.contains_key(&target)
                || self.missing_script_link_targets.contains(&target)
            {
                continue;
            }

            let entity_type = self
                .resolve_script_link_path(&link)
                .ok()
                .and_then(|path| EntityDocument::load(&path).ok())
                .map(|document| document.metadata.entity_type.trim().to_ascii_lowercase());

            if let Some(entity_type) = entity_type {
                self.script_link_target_types.insert(target, entity_type);
            } else {
                self.missing_script_link_targets.insert(target);
            }
        }
    }

    pub(crate) fn open_script_link_at(&mut self, position: Position) -> bool {
        let Some(link) = self.script_link_at(position).cloned() else {
            return false;
        };

        match self.resolve_script_link_path(&link) {
            Ok(path) => {
                let metadata_warning = EntityDocument::load(&path).err();
                if self.navigate_to_path(path.clone())
                    && let Some(error) = metadata_warning
                {
                    self.status_message = format!(
                        "Loaded {} with metadata warning: {error}",
                        status_path_label(&path)
                    );
                }
            }
            Err(message) => {
                self.status_message = message;
            }
        }

        true
    }

    pub(crate) fn document_navigation_entry(&self) -> DocumentNavigationEntry {
        DocumentNavigationEntry {
            path: self.paths.load_path.clone(),
            cursor: self.cursor,
            selection_anchor: self.selection_anchor,
            top_line: self.top_line,
            processed_top_line: self.processed_top_line,
            processed_top_visual: self.processed_top_visual,
            plain_horizontal_scroll: self.plain_horizontal_scroll,
            processed_horizontal_scroll: self.processed_horizontal_scroll,
            processed_zoom_anchor_bias_px: self.processed_zoom_anchor_bias_px,
            display_mode: self.display_mode,
            focused_panel: self.focused_panel,
            zoom: self.zoom,
            canvas_pan: self.canvas_pan,
        }
    }

    pub(crate) fn navigate_to_path(&mut self, path: PathBuf) -> bool {
        if workspace_paths_match(&path, &self.paths.load_path) {
            return self.load_from_path(path);
        }

        let previous = self.document_navigation_entry();
        if !self.load_from_path(path) {
            return false;
        }

        push_document_navigation_history(&mut self.document_navigation_history, previous);
        self.document_navigation_forward_history.clear();
        true
    }

    pub(crate) fn navigate_back(&mut self) -> bool {
        let current = self.document_navigation_entry();
        while let Some(previous) = self.document_navigation_history.pop() {
            if !self.restore_document_navigation_entry(previous, "Back to") {
                continue;
            }

            push_document_navigation_history(
                &mut self.document_navigation_forward_history,
                current,
            );
            return true;
        }

        self.status_message = "No previous page.".to_string();
        false
    }

    pub(crate) fn navigate_forward(&mut self) -> bool {
        let current = self.document_navigation_entry();
        while let Some(next) = self.document_navigation_forward_history.pop() {
            if !self.restore_document_navigation_entry(next, "Forward to") {
                continue;
            }

            push_document_navigation_history(&mut self.document_navigation_history, current);
            return true;
        }

        self.status_message = "No next page.".to_string();
        false
    }

    pub(crate) fn restore_document_navigation_entry(
        &mut self,
        entry: DocumentNavigationEntry,
        status_prefix: &str,
    ) -> bool {
        let label = status_path_label(&entry.path);
        if !self.load_from_path(entry.path) {
            return false;
        }

        self.set_zoom(entry.zoom);
        self.cursor = entry.cursor;
        self.cursor.position = self.document.clamp_position(self.cursor.position);
        self.cursor.preferred_column = self
            .cursor
            .preferred_column
            .min(self.document.line_len_chars(self.cursor.position.line));
        self.selection_anchor = entry
            .selection_anchor
            .map(|position| self.document.clamp_position(position));
        self.top_line = entry.top_line;
        self.processed_top_line = entry.processed_top_line;
        self.processed_top_visual = entry.processed_top_visual;
        self.plain_horizontal_scroll = entry.plain_horizontal_scroll;
        self.processed_horizontal_scroll = entry.processed_horizontal_scroll;
        self.processed_zoom_anchor_bias_px = entry.processed_zoom_anchor_bias_px;
        self.display_mode = entry.display_mode;
        self.focused_panel = entry.focused_panel;
        self.canvas_pan = entry.canvas_pan;
        self.clamp_processed_top_line();
        self.reset_blink();
        self.status_message = format!("{status_prefix} {label}");
        true
    }

    pub(crate) fn resolve_script_link_path(&self, link: &ScriptLink) -> Result<PathBuf, String> {
        match self.resolve_script_target_path(&link.target) {
            Ok(path) => Ok(path),
            Err(target_error) => {
                let can_resolve_by_mention =
                    link.syntax == ScriptLinkSyntax::TargetOnly && link.label != link.target;
                if !can_resolve_by_mention {
                    return Err(target_error);
                }

                self.resolve_script_mention_path(&link.label)
                    .map_err(|_| target_error)
            }
        }
    }

    pub(crate) fn resolve_script_target_path(&self, target: &str) -> Result<PathBuf, String> {
        if !is_valid_target_key(target) {
            return Err(format!("Invalid link target `{target}`."));
        }

        let mut candidates = BTreeSet::<PathBuf>::new();
        if let Some(parent) = self.paths.load_path.parent() {
            candidates.insert(parent.join(format!("{target}.md")));
            candidates.insert(parent.join(format!("{target}.markdown")));
        }
        for entry in &self.workspace_files {
            if is_matching_link_target_file(&entry.path, target) {
                candidates.insert(entry.path.clone());
            }
        }

        let existing = candidates
            .into_iter()
            .filter(|path| path.is_file())
            .map(|path| path.canonicalize().unwrap_or(path))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        match existing.as_slice() {
            [] => {
                let scaffold = scaffold_entity(self.default_script_link_root(), target)
                    .map_err(|error| format!("Unresolved link `{target}`: {error}"))?;
                Err(format!(
                    "Unresolved link `{target}`. No canonical file found. Scaffold {}.",
                    status_path_label(&scaffold.path)
                ))
            }
            [path] => Ok(path.clone()),
            many => Err(format!(
                "Ambiguous link `{target}`. Multiple canonical files match: {}.",
                many.iter()
                    .map(|path| status_path_label(path))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    pub(crate) fn resolve_script_mention_path(&self, mention: &str) -> Result<PathBuf, String> {
        let lookup = normalize_script_mention(mention);
        if lookup.is_empty() {
            return Err(format!("Unresolved mention `{mention}`."));
        }

        let existing = self
            .script_entity_candidate_files()
            .into_iter()
            .filter_map(|path| {
                let document = EntityDocument::load(&path).ok()?;
                entity_document_matches_mention(&document, &lookup)
                    .then(|| path.canonicalize().unwrap_or(path))
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        match existing.as_slice() {
            [] => Err(format!("No entity name or alias matches `{mention}`.")),
            [path] => Ok(path.clone()),
            many => Err(format!(
                "Ambiguous mention `{mention}`. Multiple entity files match: {}.",
                many.iter()
                    .map(|path| status_path_label(path))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    pub(crate) fn script_entity_candidate_files(&self) -> BTreeSet<PathBuf> {
        let mut candidates = BTreeSet::<PathBuf>::new();
        if let Some(parent) = self.paths.load_path.parent()
            && let Ok(entries) = fs::read_dir(parent)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if is_markdown_entity_file(&path) {
                    candidates.insert(path);
                }
            }
        }

        for entry in &self.workspace_files {
            if is_markdown_entity_file(&entry.path) {
                candidates.insert(entry.path.clone());
            }
        }

        candidates
    }

    pub(crate) fn default_script_link_root(&self) -> PathBuf {
        self.paths
            .load_path
            .parent()
            .map(Path::to_path_buf)
            .or_else(|| self.workspace_root.clone())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    pub(crate) fn script_link_at(&self, position: Position) -> Option<&ScriptLink> {
        self.parsed.get(position.line).and_then(|line| {
            line.script_links
                .iter()
                .find(|link| script_link_contains_visible_column(link, position.column))
        })
    }

    pub(crate) fn hovered_processed_link_at(
        &self,
        position: Position,
    ) -> Option<HoveredProcessedLink> {
        let link = self.script_link_at(position)?;
        let visible = script_link_visible_column_range(link);
        Some(HoveredProcessedLink {
            source_line: position.line,
            raw_start_column: *visible.start(),
            raw_end_column: visible.end().saturating_add(1),
        })
    }
}

pub(crate) fn push_document_navigation_history(
    history: &mut Vec<DocumentNavigationEntry>,
    entry: DocumentNavigationEntry,
) {
    if history.len() == DOCUMENT_NAVIGATION_HISTORY_LIMIT {
        history.remove(0);
    }
    history.push(entry);
}

pub(crate) fn handle_document_navigation_history(
    keys: Res<ButtonInput<KeyCode>>,
    autocomplete_capture: Res<LinkAutocompleteInputCapture>,
    middle_autoscroll: Res<MiddleAutoscrollState>,
    mut navigation_capture: ResMut<DocumentNavigationInputCapture>,
    mut state: ResMut<EditorState>,
) {
    navigation_capture.captured = false;
    let back_pressed = keys.just_pressed(KeyCode::Escape);
    let forward_pressed = shortcut_just_pressed(
        &keys,
        state.keybinds.binding(ShortcutAction::NavigateForward),
    );
    if (!back_pressed && !forward_pressed)
        || autocomplete_capture.is_captured()
        || state.link_autocomplete_has_visible_suggestions()
        || state.workspace_prompt.is_some()
        || state.command_menu.is_some()
        || state.markdown_metadata_input_active()
        || state.story_query_sheet.open
        || state.workspace_focused
        || middle_autoscroll.is_active()
        || (state.vim_enabled && state.vim_mode != VimMode::Normal)
        || (state.document_format == DocumentFormat::Canvas
            && state.canvas_editing_node_id.is_some())
    {
        return;
    }

    navigation_capture.captured = if back_pressed {
        state.navigate_back()
    } else {
        state.navigate_forward()
    };
}

pub(crate) fn entity_document_matches_mention(document: &EntityDocument, lookup: &str) -> bool {
    normalize_script_mention(&document.metadata.name) == lookup
        || document
            .metadata
            .aliases
            .iter()
            .any(|alias| normalize_script_mention(alias) == lookup)
}

pub(crate) fn normalize_script_mention(input: &str) -> String {
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

pub(crate) fn is_matching_link_target_file(path: &Path, target: &str) -> bool {
    if !is_markdown_entity_file(path) {
        return false;
    }

    path.file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| stem.eq_ignore_ascii_case(target))
}

pub(crate) fn is_markdown_entity_file(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    matches!(extension.as_deref(), Some("md") | Some("markdown"))
}

#[cfg(test)]
mod link_navigation_tests {
    use super::*;

    #[test]
    fn matches_link_target_files_ignoring_case() {
        assert!(is_matching_link_target_file(
            Path::new("Characters/Secondairy/Elisah.md"),
            "elisah"
        ));
        assert!(!is_matching_link_target_file(
            Path::new("Characters/Main/eoghan.md"),
            "elisah"
        ));
    }
}
#[allow(unused_imports)]
use super::*;
