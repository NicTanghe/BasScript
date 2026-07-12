use basscript_core::{
    EntityDocument, ScriptLinkSyntax, is_valid_target_key, scaffold_entity,
    script_link_contains_visible_column, script_link_visible_column_range,
};

impl EditorState {
    pub(crate) fn ctrl_click_link_word(&mut self, position: Position) -> bool {
        if self.script_link_at(position).is_some() {
            self.status_message = "That word is already linked.".to_string();
            return true;
        }
        let Some((range_start, range_end, label)) = self.document_word_at(position) else {
            self.status_message = "Ctrl-click a word to create a link.".to_string();
            return false;
        };

        if let Some(target) = self.unique_ctrl_click_link_target(&label) {
            if self.replace_document_range_with_link(range_start, range_end, &label, &target) {
                self.status_message = format!("Linked {label} to {target}.");
                return true;
            }
        }

        self.begin_linked_markdown_creation(range_start, range_end, label);
        true
    }

    fn unique_ctrl_click_link_target(&self, label: &str) -> Option<String> {
        if let Ok(path) = self.resolve_script_mention_path(label)
            && let Ok(entity) = EntityDocument::load(&path)
        {
            return Some(entity.metadata.target);
        }

        let expected_target = workspace_filename_target(label);
        let matches = self
            .workspace_files
            .iter()
            .filter(|entry| is_markdown_entity_file(&entry.path))
            .filter_map(|entry| entry.path.file_stem().and_then(|stem| stem.to_str()))
            .filter(|stem| workspace_filename_target(stem) == expected_target)
            .map(workspace_filename_target)
            .collect::<BTreeSet<_>>();
        (matches.len() == 1)
            .then(|| matches.into_iter().next())
            .flatten()
    }

    fn document_word_at(&self, position: Position) -> Option<(Position, Position, String)> {
        let line = self.document.line(position.line)?;
        let chars = line.chars().collect::<Vec<_>>();
        let (start, end) = linkable_word_range(line, position.column)?;
        let label = chars[start..end].iter().collect::<String>();
        Some((
            Position {
                line: position.line,
                column: start,
            },
            Position {
                line: position.line,
                column: end,
            },
            label,
        ))
    }

    pub(crate) fn replace_document_range_with_link(
        &mut self,
        range_start: Position,
        range_end: Position,
        label: &str,
        target: &str,
    ) -> bool {
        if range_start.line != range_end.line || range_start.line >= self.document.line_count() {
            return false;
        }
        let character_cue = matches!(
            self.parsed.get(range_start.line).map(|line| &line.kind),
            Some(LineKind::Character)
        );
        let replacement = document_link_replacement(label, target, character_cue);
        let snapshot = self.history_snapshot();
        let insert_at = self.document.delete_range(range_start, range_end);
        let next = self.document.insert_text(insert_at, &replacement);
        self.set_cursor(next, true);
        self.selection_anchor = None;
        self.push_undo_snapshot(snapshot);
        self.reparse_with_dirty_hint(range_start.line);
        self.close_link_autocomplete();
        true
    }

    fn begin_linked_markdown_creation(
        &mut self,
        range_start: Position,
        range_end: Position,
        label: String,
    ) {
        let Some(root) = self.workspace_root.clone() else {
            self.status_message =
                "No matching link target. Open a workspace to create one.".to_string();
            return;
        };
        let mut folders = BTreeSet::new();
        for folder in &self.workspace_folders {
            folders.insert(root.join(&folder.folder_key));
        }
        let folders = folders.into_iter().collect::<Vec<_>>();
        let source_parent = self.paths.load_path.parent();
        let selected_folder = source_parent
            .and_then(|parent| {
                folders
                    .iter()
                    .position(|folder| workspace_paths_match(folder, parent))
            })
            .unwrap_or(0);
        let mut expanded_folders = BTreeSet::new();
        if let Some(selected_path) = folders.get(selected_folder) {
            let mut ancestor = selected_path.parent();
            while let Some(path) = ancestor {
                if workspace_paths_match(path, &root) {
                    break;
                }
                if path.starts_with(&root) {
                    expanded_folders.insert(path.to_path_buf());
                }
                ancestor = path.parent();
            }
        }
        let templates = collect_workspace_markdown_templates().unwrap_or_else(|error| {
            warn!("[links] Failed reading Markdown templates: {error}");
            Vec::new()
        });
        let template_hint = matches!(
            self.parsed.get(range_start.line).map(|line| &line.kind),
            Some(LineKind::Character)
        )
        .then(|| "character".to_string());
        let filename = format!("{}.md", workspace_filename_target(&label));
        self.close_link_autocomplete();
        self.workspace_prompt = Some(WorkspacePrompt::CreateLinkedMarkdown {
            source_path: self.paths.load_path.clone(),
            range_start,
            range_end,
            label,
            filename,
            folders,
            selected_folder,
            expanded_folders,
            templates,
            template_hint,
        });
        self.status_message = "No unique target found; choose where to create it.".to_string();
    }

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

pub(crate) fn is_linkable_word_character(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_' | '-' | '\'')
}

pub(crate) fn linkable_word_range(line: &str, column: usize) -> Option<(usize, usize)> {
    let chars = line.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        return None;
    }
    let mut column = column.min(chars.len().saturating_sub(1));
    if !is_linkable_word_character(chars[column])
        && column > 0
        && is_linkable_word_character(chars[column - 1])
    {
        column -= 1;
    }
    if !is_linkable_word_character(chars[column]) {
        return None;
    }
    let mut start = column;
    while start > 0 && is_linkable_word_character(chars[start - 1]) {
        start -= 1;
    }
    let mut end = column + 1;
    while end < chars.len() && is_linkable_word_character(chars[end]) {
        end += 1;
    }
    Some((start, end))
}

pub(crate) fn document_link_replacement(label: &str, target: &str, character_cue: bool) -> String {
    let uppercase_character = character_cue
        && label.chars().any(|ch| ch.is_alphabetic())
        && label
            .chars()
            .filter(|ch| ch.is_alphabetic())
            .all(char::is_uppercase);
    if !uppercase_character && link_autocomplete_target_only_is_safe(label, target) {
        format!("[{label}]")
    } else {
        format!("[{label}]({target})")
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

    #[test]
    fn ctrl_click_word_range_keeps_name_punctuation() {
        assert_eq!(
            linkable_word_range("Meet O'Connor-Smith now", 9),
            Some((5, 19))
        );
        assert_eq!(linkable_word_range("Eoghan walks", 6), Some((0, 6)));
        assert_eq!(linkable_word_range("   ", 1), None);
    }

    #[test]
    fn uppercase_character_cues_keep_display_case_and_use_canonical_target() {
        assert_eq!(
            document_link_replacement("EOGHAN", "eoghan", true),
            "[EOGHAN](eoghan)"
        );
        assert_eq!(
            document_link_replacement("Eoghan", "eoghan", false),
            "[Eoghan]"
        );
    }
}
#[allow(unused_imports)]
use super::*;
