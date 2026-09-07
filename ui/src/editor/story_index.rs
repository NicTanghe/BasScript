use basscript_core::{
    StoryIndexDatabase, StoryIndexError, StoryIndexOpenReport, StoryIndexOpenStatus,
    StoryIndexScanReport, story_index_database_path,
};
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

#[derive(Resource, Default)]
pub(crate) struct StoryIndexTask {
    pub(crate) in_flight: Option<Task<StoryIndexTaskResult>>,
}

pub(crate) struct StoryIndexTaskResult {
    pub(crate) workspace_root: PathBuf,
    pub(crate) open_result: Result<StoryIndexOpenReport, StoryIndexError>,
    pub(crate) scan_result: Option<Result<StoryIndexScanReport, StoryIndexError>>,
}

pub(crate) fn spawn_story_index_refresh(workspace_root: PathBuf, task_state: &mut StoryIndexTask) {
    let pool = AsyncComputeTaskPool::get();
    let root = workspace_root.clone();
    let task = pool.spawn(async move {
        let open_result = StoryIndexDatabase::open_workspace(&root);
        let scan_result = match &open_result {
            Ok(report) => Some(report.database.scan_workspace_files()),
            Err(_) => None,
        };
        StoryIndexTaskResult {
            workspace_root: root,
            open_result,
            scan_result,
        }
    });
    task_state.in_flight = Some(task);
}

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
    pub(crate) fn apply_story_index_result(&mut self, result: StoryIndexTaskResult) -> String {
        let (message, index) = match result.open_result {
            Ok(report) => {
                let scan_ok = result
                    .scan_result
                    .as_ref()
                    .and_then(|scan| scan.as_ref().ok());
                let message = story_index_status_message(&report, scan_ok);
                let scan_is_ok = scan_ok.is_some();
                let index = EditorStoryIndex {
                    database: scan_is_ok.then(|| report.database.clone()),
                    workspace_root: report.database.workspace_root().to_path_buf(),
                    database_path: report.database.database_path().to_path_buf(),
                    status: if scan_is_ok {
                        editor_story_index_status(&report.status)
                    } else {
                        EditorStoryIndexStatus::Failed
                    },
                    file_count: scan_ok.map(|scan| scan.file_count).unwrap_or(0),
                    entity_count: scan_ok.map(|scan| scan.entity_count).unwrap_or(0),
                    entity_error_count: scan_ok.map(|scan| scan.entity_error_count).unwrap_or(0),
                    scene_count: scan_ok.map(|scan| scan.scene_count).unwrap_or(0),
                    appearance_count: scan_ok.map(|scan| scan.appearance_count).unwrap_or(0),
                };
                match &result.scan_result {
                    Some(Ok(_)) => info!("[story-index] {message}"),
                    Some(Err(error)) => warn!("[story-index] {message} Scan failed: {error}"),
                    None => warn!("[story-index] {message}"),
                }
                (message, index)
            }
            Err(error) => {
                let database_path = story_index_database_path(&result.workspace_root);
                let message = format!("Story index failed at {}: {error}", database_path.display());
                let index = EditorStoryIndex {
                    database: None,
                    workspace_root: result.workspace_root,
                    database_path,
                    status: EditorStoryIndexStatus::Failed,
                    file_count: 0,
                    entity_count: 0,
                    entity_error_count: 0,
                    scene_count: 0,
                    appearance_count: 0,
                };
                warn!("[story-index] {message}");
                (message, index)
            }
        };

        self.story_index = Some(index);
        message
    }

    pub(crate) fn open_story_index_for_workspace(&mut self, workspace_root: &Path) -> String {
        let open_result = StoryIndexDatabase::open_workspace(workspace_root);
        let scan_result = match &open_result {
            Ok(report) => Some(report.database.scan_workspace_files()),
            Err(_) => None,
        };
        self.apply_story_index_result(StoryIndexTaskResult {
            workspace_root: workspace_root.to_path_buf(),
            open_result,
            scan_result,
        })
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

pub(crate) fn poll_story_index_task(
    mut state: ResMut<EditorState>,
    mut task_state: ResMut<StoryIndexTask>,
    mut redraw: MessageWriter<bevy::window::RequestRedraw>,
) {
    let Some(mut task) = task_state.in_flight.take() else {
        return;
    };

    if let Some(result) = block_on(poll_once(&mut task)) {
        if state.workspace_root.as_deref() == Some(&result.workspace_root) {
            let message = state.apply_story_index_result(result);
            state.status_message = message;
            state.workspace_ui_dirty = true;
            redraw.write(bevy::window::RequestRedraw);
        } else {
            info!(
                "[story-index] Discarded index result for {} as active workspace changed",
                result.workspace_root.display()
            );
        }
    } else {
        task_state.in_flight = Some(task);
        redraw.write(bevy::window::RequestRedraw);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_story_index_result_handles_error() {
        let mut world = World::new();
        let mut state = EditorState::from_world(&mut world);
        let path = PathBuf::from("/nonexistent/test/workspace");
        let result = StoryIndexTaskResult {
            workspace_root: path.clone(),
            open_result: Err(StoryIndexError::Io(std::io::Error::other("test error"))),
            scan_result: None,
        };

        let message = state.apply_story_index_result(result);
        assert!(message.contains("Story index failed"));
        assert!(state.story_index.is_some());
        let index = state.story_index.as_ref().unwrap();
        assert_eq!(index.workspace_root, path);
        assert!(matches!(index.status, EditorStoryIndexStatus::Failed));
    }

    #[test]
    fn poll_story_index_task_applies_completed_result() {
        let mut app = App::new();
        app.init_resource::<EditorState>()
            .init_resource::<StoryIndexTask>()
            .add_message::<bevy::window::RequestRedraw>()
            .add_systems(Update, poll_story_index_task);

        let test_root = PathBuf::from("/test/workspace/root");
        {
            let mut state = app.world_mut().resource_mut::<EditorState>();
            state.workspace_root = Some(test_root.clone());
        }

        let task = bevy::tasks::AsyncComputeTaskPool::get_or_init(bevy::tasks::TaskPool::default)
            .spawn(async move {
                StoryIndexTaskResult {
                    workspace_root: PathBuf::from("/test/workspace/root"),
                    open_result: Err(StoryIndexError::Io(std::io::Error::other("test error"))),
                    scan_result: None,
                }
            });

        {
            let mut task_state = app.world_mut().resource_mut::<StoryIndexTask>();
            task_state.in_flight = Some(task);
        }

        app.update();

        let state = app.world().resource::<EditorState>();
        assert!(state.story_index.is_some());
        assert_eq!(
            state.story_index.as_ref().unwrap().workspace_root,
            test_root
        );
        let task_state = app.world().resource::<StoryIndexTask>();
        assert!(task_state.in_flight.is_none());
    }
}
