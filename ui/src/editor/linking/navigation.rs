use basscript_core::{
    EntityDocument, is_valid_target_key, scaffold_entity, script_link_contains_visible_column,
    script_link_visible_column_range,
};

impl EditorState {
    fn clear_script_link_target_cache(&mut self) {
        self.script_link_target_types.clear();
        self.missing_script_link_targets.clear();
    }

    fn ensure_current_script_link_targets_cached(&mut self) {
        let targets = self
            .parsed
            .iter()
            .flat_map(|line| line.script_links.iter().map(|link| link.target.clone()))
            .collect::<BTreeSet<_>>();

        self.script_link_target_types
            .retain(|target, _| targets.contains(target));
        self.missing_script_link_targets
            .retain(|target| targets.contains(target));

        for target in targets {
            if self.script_link_target_types.contains_key(&target)
                || self.missing_script_link_targets.contains(&target)
            {
                continue;
            }

            let entity_type = self
                .resolve_script_target_path(&target)
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

    fn open_script_link_at(&mut self, position: Position) -> bool {
        let Some(target) = self.script_link_target_at(position).map(str::to_string) else {
            return false;
        };

        match self.resolve_script_target_path(&target) {
            Ok(path) => {
                let metadata_warning = EntityDocument::load(&path).err();
                self.load_from_path(path.clone());
                if let Some(error) = metadata_warning {
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

    fn resolve_script_target_path(&self, target: &str) -> Result<PathBuf, String> {
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

    fn default_script_link_root(&self) -> PathBuf {
        self.paths
            .load_path
            .parent()
            .map(Path::to_path_buf)
            .or_else(|| self.workspace_root.clone())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn script_link_at(&self, position: Position) -> Option<&ScriptLink> {
        self.parsed.get(position.line).and_then(|line| {
            line.script_links
                .iter()
                .find(|link| script_link_contains_visible_column(link, position.column))
        })
    }

    fn script_link_target_at(&self, position: Position) -> Option<&str> {
        self.script_link_at(position).map(|link| link.target.as_str())
    }

    fn hovered_processed_link_at(&self, position: Position) -> Option<HoveredProcessedLink> {
        let link = self.script_link_at(position)?;
        let visible = script_link_visible_column_range(link);
        Some(HoveredProcessedLink {
            source_line: position.line,
            raw_start_column: *visible.start(),
            raw_end_column: visible.end().saturating_add(1),
        })
    }
}

fn is_matching_link_target_file(path: &Path, target: &str) -> bool {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    let stem = path.file_stem().and_then(|stem| stem.to_str());

    stem == Some(target) && matches!(extension.as_deref(), Some("md") | Some("markdown"))
}
