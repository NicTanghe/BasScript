use basscript_core::{
    StoryIndexDatabase, StoryIndexOpenReport, StoryIndexOpenStatus, story_index_database_path,
};

#[derive(Clone, Debug)]
struct EditorStoryIndex {
    workspace_root: PathBuf,
    database_path: PathBuf,
    status: EditorStoryIndexStatus,
}

#[derive(Clone, Debug)]
enum EditorStoryIndexStatus {
    Ready,
    Created,
    Recreated,
    Failed,
}

impl EditorStoryIndex {
    fn visible_label(&self) -> String {
        let workspace = self
            .workspace_root
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("workspace");
        let database = self
            .database_path
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("story-index.sqlite3");
        format!(" | index: {} ({workspace}/{database})", self.status.label())
    }
}

impl EditorStoryIndexStatus {
    fn label(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Created => "created",
            Self::Recreated => "rebuilt",
            Self::Failed => "failed",
        }
    }
}

impl EditorState {
    fn open_story_index_for_workspace(&mut self, workspace_root: &Path) -> String {
        match StoryIndexDatabase::open_workspace(workspace_root) {
            Ok(report) => {
                let message = story_index_status_message(&report);
                self.story_index = Some(EditorStoryIndex {
                    workspace_root: report.database.workspace_root().to_path_buf(),
                    database_path: report.database.database_path().to_path_buf(),
                    status: editor_story_index_status(&report.status),
                });
                info!("[story-index] {message}");
                message
            }
            Err(error) => {
                let database_path = story_index_database_path(workspace_root);
                let message = format!("Story index failed at {}: {error}", database_path.display());
                self.story_index = Some(EditorStoryIndex {
                    workspace_root: workspace_root.to_path_buf(),
                    database_path,
                    status: EditorStoryIndexStatus::Failed,
                });
                warn!("[story-index] {message}");
                message
            }
        }
    }

    fn story_index_visible_label(&self) -> String {
        self.story_index
            .as_ref()
            .map(EditorStoryIndex::visible_label)
            .unwrap_or_default()
    }
}

fn editor_story_index_status(status: &StoryIndexOpenStatus) -> EditorStoryIndexStatus {
    match status {
        StoryIndexOpenStatus::Created => EditorStoryIndexStatus::Created,
        StoryIndexOpenStatus::Ready => EditorStoryIndexStatus::Ready,
        StoryIndexOpenStatus::Recreated { .. } => EditorStoryIndexStatus::Recreated,
    }
}

fn story_index_status_message(report: &StoryIndexOpenReport) -> String {
    match &report.status {
        StoryIndexOpenStatus::Created => {
            format!(
                "Story index created at {}.",
                report.database.database_path().display()
            )
        }
        StoryIndexOpenStatus::Ready => {
            format!("Story index ready at {}.", report.database.database_path().display())
        }
        StoryIndexOpenStatus::Recreated {
            reason,
            previous_database_path,
        } => {
            let previous = previous_database_path
                .as_ref()
                .map(|path| format!(" Previous database moved to {}.", path.display()))
                .unwrap_or_default();
            format!(
                "Story index rebuilt at {} after {reason}.{previous}",
                report.database.database_path().display()
            )
        }
    }
}
