use basscript_core::{
    EntityDocument, is_valid_target_key, scaffold_entity, script_link_contains_visible_column,
};

impl EditorState {
    fn open_script_link_at(&mut self, position: Position) -> bool {
        let Some(target) = self
            .parsed
            .get(position.line)
            .and_then(|line| {
                line.script_links
                    .iter()
                    .find(|link| script_link_contains_visible_column(link, position.column))
            })
            .map(|link| link.target.clone())
        else {
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
}

fn is_matching_link_target_file(path: &Path, target: &str) -> bool {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    let stem = path.file_stem().and_then(|stem| stem.to_str());

    stem == Some(target) && matches!(extension.as_deref(), Some("md") | Some("markdown"))
}
