use basscript_core::{
    StoryIndexDatabase, StoryIndexOpenReport, StoryIndexOpenStatus, StoryIndexScanReport,
    story_index_database_path,
};

#[derive(Clone, Debug)]
pub(crate) struct EditorStoryIndex {
    pub(crate) database: Option<StoryIndexDatabase>,
    pub(crate) workspace_root: PathBuf,
    pub(crate) database_path: PathBuf,
    pub(crate) status: EditorStoryIndexStatus,
    pub(crate) file_count: usize,
    pub(crate) entity_count: usize,
    pub(crate) entity_error_count: usize,
    pub(crate) scene_count: usize,
    pub(crate) appearance_count: usize,
}

#[derive(Clone, Debug)]
pub(crate) enum EditorStoryIndexStatus {
    Ready,
    Created,
    Recreated,
    Failed,
}

impl EditorStoryIndex {
    pub(crate) fn visible_label(&self) -> String {
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
        format!(
            " | index: {} {} files, {} entities, {} entity errors, {} scenes, {} appearances ({workspace}/{database})",
            self.status.label(),
            self.file_count,
            self.entity_count,
            self.entity_error_count,
            self.scene_count,
            self.appearance_count
        )
    }
}

impl EditorStoryIndexStatus {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Created => "created",
            Self::Recreated => "rebuilt",
            Self::Failed => "failed",
        }
    }
}

impl EditorState {
    pub(crate) fn open_story_index_for_workspace(&mut self, workspace_root: &Path) -> String {
        match StoryIndexDatabase::open_workspace(workspace_root) {
            Ok(report) => {
                let scan = report.database.scan_workspace_files();
                let message = story_index_status_message(&report, scan.as_ref().ok());
                self.story_index = Some(EditorStoryIndex {
                    database: scan.is_ok().then(|| report.database.clone()),
                    workspace_root: report.database.workspace_root().to_path_buf(),
                    database_path: report.database.database_path().to_path_buf(),
                    status: if scan.is_ok() {
                        editor_story_index_status(&report.status)
                    } else {
                        EditorStoryIndexStatus::Failed
                    },
                    file_count: scan.as_ref().map(|scan| scan.file_count).unwrap_or(0),
                    entity_count: scan.as_ref().map(|scan| scan.entity_count).unwrap_or(0),
                    entity_error_count: scan
                        .as_ref()
                        .map(|scan| scan.entity_error_count)
                        .unwrap_or(0),
                    scene_count: scan.as_ref().map(|scan| scan.scene_count).unwrap_or(0),
                    appearance_count: scan.as_ref().map(|scan| scan.appearance_count).unwrap_or(0),
                });
                match scan {
                    Ok(_) => info!("[story-index] {message}"),
                    Err(error) => warn!("[story-index] {message} Scan failed: {error}"),
                }
                message
            }
            Err(error) => {
                let database_path = story_index_database_path(workspace_root);
                let message = format!("Story index failed at {}: {error}", database_path.display());
                self.story_index = Some(EditorStoryIndex {
                    database: None,
                    workspace_root: workspace_root.to_path_buf(),
                    database_path,
                    status: EditorStoryIndexStatus::Failed,
                    file_count: 0,
                    entity_count: 0,
                    entity_error_count: 0,
                    scene_count: 0,
                    appearance_count: 0,
                });
                warn!("[story-index] {message}");
                message
            }
        }
    }

    pub(crate) fn refresh_story_index_for_workspace(&mut self) -> Option<String> {
        let workspace_root = self.workspace_root.clone()?;
        Some(self.open_story_index_for_workspace(&workspace_root))
    }

    pub(crate) fn story_index_visible_label(&self) -> String {
        self.story_index
            .as_ref()
            .map(EditorStoryIndex::visible_label)
            .unwrap_or_default()
    }
}

pub(crate) fn editor_story_index_status(status: &StoryIndexOpenStatus) -> EditorStoryIndexStatus {
    match status {
        StoryIndexOpenStatus::Created => EditorStoryIndexStatus::Created,
        StoryIndexOpenStatus::Ready => EditorStoryIndexStatus::Ready,
        StoryIndexOpenStatus::Recreated { .. } => EditorStoryIndexStatus::Recreated,
    }
}

pub(crate) fn story_index_status_message(
    report: &StoryIndexOpenReport,
    scan: Option<&StoryIndexScanReport>,
) -> String {
    let scan_summary = scan
        .map(story_index_scan_summary)
        .unwrap_or_else(|| "Index failed.".to_string());

    match &report.status {
        StoryIndexOpenStatus::Created => {
            format!(
                "Story index created at {}. {scan_summary}",
                report.database.database_path().display(),
            )
        }
        StoryIndexOpenStatus::Ready => {
            format!(
                "Story index ready at {}. {scan_summary}",
                report.database.database_path().display()
            )
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
                "Story index rebuilt at {} after {reason}.{previous} {scan_summary}",
                report.database.database_path().display()
            )
        }
    }
}

pub(crate) fn story_index_scan_summary(scan: &StoryIndexScanReport) -> String {
    format!(
        "Index ready: {} files (+{}, ~{}, -{}), {} entities, {} aliases, {} entity errors, {} scenes, {} appearances.",
        scan.file_count,
        scan.inserted_count,
        scan.updated_count,
        scan.removed_count,
        scan.entity_count,
        scan.entity_alias_count,
        scan.entity_error_count,
        scan.scene_count,
        scan.appearance_count
    )
}
#[allow(unused_imports)]
use super::*;
