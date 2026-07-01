const WORKSPACE_ROOT_LABEL_EMPTY: &str = "No workspace opened.";
const WORKSPACE_ROOT_LABEL_PREFIX: &str = "";
const WORKSPACE_EMPTY_RESULTS_LABEL: &str = "No .fountain/.md/.txt/.canvas files found.";

// pretty sure these arent really working

// Horizontal gap between explorer left wall and the root label line.
const WORKSPACE_ROOT_LABEL_LEFT_MARGIN: f32 = 0.0;
// Horizontal gap between explorer left wall and folder/file tree rows.
const WORKSPACE_TREE_LIST_LEFT_MARGIN: f32 = 0.0;
const WORKSPACE_TREE_DEPTH_INDENT: f32 = 14.0;
const WORKSPACE_FILE_ROW_EXTRA_LEFT: f32 = 2.0;
const WORKSPACE_TREE_VERTICAL_PADDING: f32 = 10.0;
const WORKSPACE_TREE_ROW_HEIGHT: f32 = 24.0;
const WORKSPACE_TREE_ROW_GAP: f32 = 4.0;
const WORKSPACE_TREE_WHEEL_LINE_PX: f32 = 36.0;

// Set this to Some("C:/path/to/folder") to force the initial opened workspace root.
// Keep as None to use the parent directory of the currently loaded document.
const WORKSPACE_INITIAL_ROOT_OVERRIDE: Option<&str> = None;

#[derive(Component)]
struct WorkspaceRootLabel;

#[derive(Component)]
struct WorkspaceFileList;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct WorkspaceFileButton {
    index: usize,
}

#[derive(Component, Clone, Debug, PartialEq, Eq)]
struct WorkspaceFolderToggleButton {
    folder_key: String,
}

#[derive(Resource, Clone)]
struct WorkspaceIcons {
    folder_closed: Handle<Image>,
    folder_open: Handle<Image>,
}

#[derive(Clone, Debug)]
struct WorkspaceFileEntry {
    path: PathBuf,
    relative_display: String,
}

#[derive(Clone, Debug)]
struct WorkspaceFolderEntry {
    folder_key: String,
    folder_name: String,
    parent_key: String,
}

#[derive(Clone, Debug, Default)]
struct WorkspaceEntries {
    folders: Vec<WorkspaceFolderEntry>,
    files: Vec<WorkspaceFileEntry>,
}

#[derive(Clone, Debug)]
enum WorkspaceSidebarRow {
    Folder {
        folder_key: String,
        folder_name: String,
        depth: usize,
        expanded: bool,
    },
    File {
        file_index: usize,
        file_name: String,
        depth: usize,
    },
}

fn apply_initial_workspace_root(
    state: &mut EditorState,
    initial_status: &str,
    saved_workspace_root: Option<&str>,
) {
    let Some(root) = resolve_initial_workspace_root(&state.paths.load_path, saved_workspace_root) else {
        return;
    };

    state.set_workspace_root(root);
    state.status_message = initial_status.to_string();
}

fn resolve_initial_workspace_root(load_path: &Path, saved_workspace_root: Option<&str>) -> Option<PathBuf> {
    WORKSPACE_INITIAL_ROOT_OVERRIDE
        .map(PathBuf::from)
        .or_else(|| saved_workspace_root.map(PathBuf::from))
        .or_else(|| load_path.parent().map(Path::to_path_buf))
}

fn workspace_root_label_text(root: Option<&Path>) -> String {
    root.map_or_else(
        || WORKSPACE_ROOT_LABEL_EMPTY.to_string(),
        |root| {
            let folder_name = root
                .file_name()
                .and_then(|name| name.to_str())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| root.to_str().unwrap_or(WORKSPACE_ROOT_LABEL_EMPTY));
            format!("{WORKSPACE_ROOT_LABEL_PREFIX}{folder_name}")
        },
    )
}

impl EditorState {
    fn set_workspace_root(&mut self, root: PathBuf) {
        let normalized_root = root.canonicalize().unwrap_or(root);
        self.workspace_root = Some(normalized_root.clone());
        self.workspace_active_file = None;
        self.workspace_selected_row = None;
        self.clear_script_link_target_cache();

        match collect_workspace_entries(&normalized_root) {
            Ok(entries) => {
                self.workspace_folders = entries.folders;
                self.workspace_files = entries.files;
                self.workspace_expanded_folders =
                    default_expanded_workspace_folders(&self.workspace_folders, &self.workspace_files);
                self.sync_workspace_active_file();
                self.normalize_workspace_selected_row();
                let story_index_status = self.open_story_index_for_workspace(&normalized_root);
                self.status_message = format!(
                    "Opened workspace {} ({} files). {}",
                    normalized_root.display(),
                    self.workspace_files.len(),
                    story_index_status
                );
            }
            Err(error) => {
                self.workspace_folders.clear();
                self.workspace_files.clear();
                self.workspace_active_file = None;
                self.workspace_selected_row = None;
                self.workspace_expanded_folders.clear();
                self.story_index = None;
                self.status_message = format!(
                    "Workspace scan failed for {}: {error}",
                    normalized_root.display()
                );
            }
        }

        self.workspace_ui_dirty = true;

        let persistent = persistent_settings_from_state(self);
        if let Err(error) = save_persistent_settings(&persistent) {
            warn!("[settings] Failed saving workspace root after open: {error}");
        }
    }

    fn open_workspace_file(&mut self, index: usize) {
        let Some(entry) = self.workspace_files.get(index) else {
            self.status_message = "Workspace file selection is out of range.".to_string();
            return;
        };

        self.load_from_path(entry.path.clone());
    }

    fn refresh_workspace(&mut self) {
        let Some(root) = self.workspace_root.clone() else {
            self.workspace_folders.clear();
            self.workspace_files.clear();
            self.workspace_active_file = None;
            self.workspace_selected_row = None;
            self.workspace_ui_dirty = true;
            return;
        };

        match collect_workspace_entries(&root) {
            Ok(entries) => {
                self.workspace_folders = entries.folders;
                self.workspace_files = entries.files;
                self.sync_workspace_active_file();
                self.normalize_workspace_selected_row();
                let story_index_status = self.refresh_story_index_for_workspace();
                self.workspace_ui_dirty = true;
                if let Some(story_index_status) = story_index_status {
                    self.status_message = story_index_status;
                }
            }
            Err(error) => {
                self.status_message = format!("Workspace refresh failed for {}: {error}", root.display());
            }
        }
    }

    fn refresh_workspace_after_path_change(&mut self) {
        let Some(root) = self.workspace_root.as_ref() else {
            return;
        };
        if workspace_path_is_under_root(root, &self.paths.load_path)
            || workspace_path_is_under_root(root, &self.paths.save_path)
        {
            self.refresh_workspace();
        } else {
            self.sync_workspace_active_file();
        }
    }

    fn toggle_workspace_folder(&mut self, folder_key: &str) {
        if self.workspace_expanded_folders.contains(folder_key) {
            self.workspace_expanded_folders.remove(folder_key);
        } else {
            self.workspace_expanded_folders
                .insert(folder_key.to_owned());
        }
        self.workspace_ui_dirty = true;
    }

    fn sync_workspace_active_file(&mut self) {
        self.workspace_active_file = self
            .workspace_files
            .iter()
            .position(|entry| workspace_paths_match(&entry.path, &self.paths.load_path));
        if self.workspace_selected_row.is_none()
            && let Some(index) = self.workspace_active_file
            && let Some(entry) = self.workspace_files.get(index)
        {
            self.workspace_selected_row = Some(WorkspaceSelectedRow::File(entry.path.clone()));
        }
        self.workspace_ui_dirty = true;
    }

    fn normalize_workspace_selected_row(&mut self) {
        if self.workspace_selected_row_exists() {
            return;
        }

        self.workspace_selected_row = self
            .workspace_active_file
            .and_then(|index| self.workspace_files.get(index))
            .map(|entry| WorkspaceSelectedRow::File(entry.path.clone()))
            .or_else(|| {
                workspace_sidebar_rows(self)
                    .into_iter()
                    .next()
                    .and_then(|row| workspace_selection_for_row(self, &row))
            });
    }

    fn workspace_selected_row_exists(&self) -> bool {
        let Some(selection) = self.workspace_selected_row.as_ref() else {
            return false;
        };
        workspace_sidebar_rows(self)
            .iter()
            .any(|row| workspace_row_matches_selection(self, row, selection))
    }
}

fn workspace_sidebar_rows(state: &EditorState) -> Vec<WorkspaceSidebarRow> {
    let mut folders_by_parent = BTreeMap::<String, Vec<(String, String)>>::new();
    let mut files_by_parent = BTreeMap::<String, Vec<(usize, String)>>::new();

    for folder in &state.workspace_folders {
        folders_by_parent
            .entry(folder.parent_key.clone())
            .or_default()
            .push((folder.folder_key.clone(), folder.folder_name.clone()));
    }

    for (index, file) in state.workspace_files.iter().enumerate() {
        let parent_key = workspace_parent_key(&file.relative_display);
        let file_name = workspace_base_name(&file.relative_display);
        files_by_parent
            .entry(parent_key)
            .or_default()
            .push((index, file_name));
    }

    for folders in folders_by_parent.values_mut() {
        folders.sort_by(|left, right| left.1.cmp(&right.1));
    }
    for files in files_by_parent.values_mut() {
        files.sort_by(|left, right| left.1.cmp(&right.1));
    }

    let mut rows = Vec::<WorkspaceSidebarRow>::new();
    append_workspace_sidebar_rows(
        "",
        0,
        &state.workspace_expanded_folders,
        &folders_by_parent,
        &files_by_parent,
        &mut rows,
    );
    rows
}

fn append_workspace_sidebar_rows(
    parent_key: &str,
    depth: usize,
    expanded_folders: &BTreeSet<String>,
    folders_by_parent: &BTreeMap<String, Vec<(String, String)>>,
    files_by_parent: &BTreeMap<String, Vec<(usize, String)>>,
    out: &mut Vec<WorkspaceSidebarRow>,
) {
    if let Some(folders) = folders_by_parent.get(parent_key) {
        for (folder_key, folder_name) in folders {
            let expanded = expanded_folders.contains(folder_key);
            out.push(WorkspaceSidebarRow::Folder {
                folder_key: folder_key.clone(),
                folder_name: folder_name.clone(),
                depth,
                expanded,
            });
            if expanded {
                append_workspace_sidebar_rows(
                    folder_key,
                    depth.saturating_add(1),
                    expanded_folders,
                    folders_by_parent,
                    files_by_parent,
                    out,
                );
            }
        }
    }

    if let Some(files) = files_by_parent.get(parent_key) {
        for (file_index, file_name) in files {
            out.push(WorkspaceSidebarRow::File {
                file_index: *file_index,
                file_name: file_name.clone(),
                depth,
            });
        }
    }
}

fn workspace_parent_key(relative_display: &str) -> String {
    relative_display
        .rsplit_once('/')
        .map_or_else(String::new, |(parent, _)| parent.to_owned())
}

fn workspace_base_name(relative_display: &str) -> String {
    relative_display
        .rsplit('/')
        .next()
        .map_or_else(String::new, str::to_owned)
}

fn workspace_sidebar_content_height(row_count: usize) -> f32 {
    if row_count == 0 {
        0.0
    } else {
        row_count as f32 * WORKSPACE_TREE_ROW_HEIGHT
            + row_count.saturating_sub(1) as f32 * WORKSPACE_TREE_ROW_GAP
    }
}

fn workspace_sidebar_row_top(index: usize) -> f32 {
    index as f32 * (WORKSPACE_TREE_ROW_HEIGHT + WORKSPACE_TREE_ROW_GAP)
}

fn workspace_sidebar_max_scroll(row_count: usize, viewport_height: f32) -> f32 {
    (workspace_sidebar_content_height(row_count) - viewport_height).max(0.0)
}

fn default_expanded_workspace_folders(
    folders: &[WorkspaceFolderEntry],
    files: &[WorkspaceFileEntry],
) -> BTreeSet<String> {
    let mut expanded = BTreeSet::<String>::new();
    for folder in folders {
        if folder.parent_key.is_empty() {
            expanded.insert(folder.folder_key.clone());
        }
    }
    for file in files {
        let Some((top_level, _)) = file.relative_display.split_once('/') else {
            continue;
        };
        expanded.insert(top_level.to_owned());
    }
    expanded
}

fn collect_workspace_entries(root: &Path) -> io::Result<WorkspaceEntries> {
    let mut folders = Vec::<WorkspaceFolderEntry>::new();
    let mut files = Vec::<WorkspaceFileEntry>::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                if should_skip_workspace_dir(&path) {
                    continue;
                }
                if let Some(folder) = workspace_folder_entry(root, &path) {
                    folders.push(folder);
                }
                stack.push(path);
                continue;
            }

            if !file_type.is_file() || !is_workspace_file_candidate(&path) {
                continue;
            }

            let relative = path
                .strip_prefix(root)
                .unwrap_or(path.as_path())
                .to_string_lossy()
                .replace('\\', "/");

            files.push(WorkspaceFileEntry {
                path,
                relative_display: relative,
            });
        }
    }

    folders.sort_by(|left, right| left.folder_key.cmp(&right.folder_key));
    files.sort_by(|left, right| left.relative_display.cmp(&right.relative_display));
    Ok(WorkspaceEntries { folders, files })
}

fn workspace_folder_entry(root: &Path, path: &Path) -> Option<WorkspaceFolderEntry> {
    let folder_key = path.strip_prefix(root).ok()?.to_string_lossy().replace('\\', "/");
    if folder_key.is_empty() {
        return None;
    }

    let folder_name = workspace_base_name(&folder_key);
    let parent_key = workspace_parent_key(&folder_key);
    Some(WorkspaceFolderEntry {
        folder_key,
        folder_name,
        parent_key,
    })
}

fn should_skip_workspace_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    name.starts_with('.') || matches!(name, "target" | "node_modules")
}

fn is_workspace_file_candidate(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    matches!(
        extension.as_deref(),
        Some("fountain") | Some("txt") | Some("md") | Some("markdown") | Some("canvas")
    )
}

fn workspace_sidebar_bundle(font: Handle<Font>, background: Color) -> impl Bundle {
    (
        Node {
            width: px(WORKSPACE_WIDTH_DEFAULT),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(8.0),
            padding: UiRect::axes(
                px(WORKSPACE_ROOT_LABEL_LEFT_MARGIN),
                px(WORKSPACE_TREE_VERTICAL_PADDING),
            ),
            ..default()
        },
        WorkspaceSidebarPane,
        BackgroundColor(background),
        children![
            (
                Text::new(WORKSPACE_ROOT_LABEL_EMPTY),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MUTED),
                WorkspaceRootLabel,
            ),
            (
                Node {
                    width: percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::new(
                        px(WORKSPACE_TREE_LIST_LEFT_MARGIN),
                        px(0.0),
                        px(0.0),
                        px(0.0),
                    ),
                    row_gap: px(WORKSPACE_TREE_ROW_GAP),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollPosition::default(),
                RelativeCursorPosition::default(),
                WorkspaceFileList,
            ),
        ],
    )
}

fn handle_workspace_file_buttons(
    interaction_query: Query<
        (&Interaction, &WorkspaceFileButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<EditorState>,
) {
    for (interaction, file_button) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(entry) = state.workspace_files.get(file_button.index) {
            state.workspace_selected_row = Some(WorkspaceSelectedRow::File(entry.path.clone()));
        }
        state.workspace_focused = true;
        state.workspace_ui_dirty = true;
        state.open_workspace_file(file_button.index);
    }
}

fn handle_workspace_folder_buttons(
    interaction_query: Query<
        (&Interaction, &WorkspaceFolderToggleButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<EditorState>,
) {
    for (interaction, folder_button) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        state.workspace_selected_row =
            Some(WorkspaceSelectedRow::Folder(folder_button.folder_key.clone()));
        state.workspace_focused = true;
        state.toggle_workspace_folder(&folder_button.folder_key);
    }
}

fn sync_workspace_sidebar(
    mut commands: Commands,
    fonts: Res<EditorFonts>,
    workspace_icons: Res<WorkspaceIcons>,
    mut state: ResMut<EditorState>,
    mut root_label_query: Query<&mut Text, With<WorkspaceRootLabel>>,
    list_query: Query<(Entity, Option<&Children>), With<WorkspaceFileList>>,
) {
    if !state.workspace_ui_dirty {
        return;
    }

    if let Ok(mut root_label) = root_label_query.single_mut() {
        **root_label = workspace_root_label_text(state.workspace_root.as_deref());
    }

    let Ok((file_list_entity, children)) = list_query.single() else {
        state.workspace_ui_dirty = false;
        return;
    };

    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    let rows = workspace_sidebar_rows(&state);
    let active_relative_display = state
        .workspace_active_file
        .and_then(|index| state.workspace_files.get(index))
        .map(|entry| entry.relative_display.as_str());
    let selected_row = state.workspace_selected_row.clone();

    commands.entity(file_list_entity).with_children(|parent| {
        if rows.is_empty() {
            parent.spawn((
                Text::new(WORKSPACE_EMPTY_RESULTS_LABEL),
                TextFont {
                    font: fonts.regular.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MUTED),
            ));
            return;
        }

        for row in rows {
            match row {
                WorkspaceSidebarRow::Folder {
                    folder_key,
                    folder_name,
                    depth,
                    expanded,
                } => {
                    let icon_handle = if expanded {
                        workspace_icons.folder_open.clone()
                    } else {
                        workspace_icons.folder_closed.clone()
                    };
                    let left_indent = depth as f32 * WORKSPACE_TREE_DEPTH_INDENT;
                    let fallback_marker = if expanded { "▾" } else { "▸" };
                    let is_selected = selected_row
                        .as_ref()
                        .is_some_and(|selection| matches!(selection, WorkspaceSelectedRow::Folder(selected_key) if selected_key == &folder_key));
                    let folder_is_opened =
                        expanded || folder_contains_active_file(active_relative_display, &folder_key);
                    let folder_font = if folder_is_opened {
                        fonts.bold.clone()
                    } else {
                        fonts.regular.clone()
                    };
                    let row_bg = if is_selected {
                        COLOR_WORKSPACE_ROW_SELECTED_BG
                    } else {
                        Color::srgba(0.0, 0.0, 0.0, 0.0)
                    };

                    parent.spawn((
                        Button,
                        WorkspaceFolderToggleButton {
                            folder_key: folder_key.clone(),
                        },
                        Node {
                            width: percent(100.0),
                            height: px(WORKSPACE_TREE_ROW_HEIGHT),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(6.0),
                            padding: UiRect::new(px(left_indent), px(0.0), px(0.0), px(0.0)),
                            ..default()
                        },
                        BackgroundColor(row_bg),
                        children![
                            (
                                Text::new(fallback_marker),
                                TextFont {
                                    font: fonts.regular.clone(),
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(COLOR_TEXT_MUTED),
                            ),
                            (
                                Node {
                                    width: px(18.0),
                                    height: px(18.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                children![(
                                    ImageNode::new(icon_handle),
                                    Node {
                                        width: px(14.0),
                                        height: px(14.0),
                                        ..default()
                                    },
                                )],
                            ),
                            (
                                Text::new(folder_name),
                                TextFont {
                                    font: folder_font,
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(COLOR_TEXT_MAIN),
                            )
                        ],
                    ));
                }
                WorkspaceSidebarRow::File {
                    file_index,
                    file_name,
                    depth,
                } => {
                    let left_indent =
                        WORKSPACE_FILE_ROW_EXTRA_LEFT + depth as f32 * WORKSPACE_TREE_DEPTH_INDENT;
                    let file_path = state
                        .workspace_files
                        .get(file_index)
                        .map(|entry| entry.path.clone());
                    let is_active = state.workspace_active_file == Some(file_index);
                    let is_selected = selected_row.as_ref().is_some_and(|selection| {
                        matches!(
                            (selection, file_path.as_ref()),
                            (WorkspaceSelectedRow::File(selected_path), Some(file_path))
                                if workspace_paths_match(selected_path, file_path)
                        )
                    });
                    let text_color = if is_active {
                        COLOR_WORKSPACE_FILE_SELECTED
                    } else {
                        COLOR_WORKSPACE_FILE
                    };
                    let row_bg = match (is_selected, is_active) {
                        (true, true) => COLOR_WORKSPACE_ROW_SELECTED_ACTIVE_BG,
                        (true, false) => COLOR_WORKSPACE_ROW_SELECTED_BG,
                        (false, true) => COLOR_WORKSPACE_ROW_ACTIVE_BG,
                        (false, false) => Color::srgba(0.0, 0.0, 0.0, 0.0),
                    };

                    parent.spawn((
                        Button,
                        WorkspaceFileButton { index: file_index },
                        Node {
                            width: percent(100.0),
                            height: px(WORKSPACE_TREE_ROW_HEIGHT),
                            align_items: AlignItems::Center,
                            padding: UiRect::new(px(left_indent), px(8.0), px(0.0), px(0.0)),
                            ..default()
                        },
                        BackgroundColor(row_bg),
                        children![(
                            Text::new(file_name),
                            TextFont {
                                font: fonts.regular.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(text_color),
                        )],
                    ));
                }
            }
        }
    });

    state.workspace_ui_dirty = false;
}

fn workspace_file_list_hovered(
    list_query: &Query<&RelativeCursorPosition, With<WorkspaceFileList>>,
) -> bool {
    list_query.iter().any(RelativeCursorPosition::cursor_over)
}

fn handle_workspace_mouse_scroll(
    mut mouse_wheels: MessageReader<MouseWheel>,
    state: Res<EditorState>,
    mut list_query: Query<
        (&RelativeCursorPosition, &ComputedNode, &mut ScrollPosition),
        With<WorkspaceFileList>,
    >,
) {
    if !state.workspace_sidebar_visible {
        return;
    }

    let Ok((relative_cursor, computed, mut scroll_position)) = list_query.single_mut() else {
        return;
    };
    if !relative_cursor.cursor_over() {
        return;
    }

    let mut delta_y = 0.0_f32;
    for wheel in mouse_wheels.read() {
        delta_y += match wheel.unit {
            MouseScrollUnit::Line => -wheel.y * WORKSPACE_TREE_WHEEL_LINE_PX,
            MouseScrollUnit::Pixel => -wheel.y,
        };
    }

    if delta_y.abs() <= f32::EPSILON {
        return;
    }

    let viewport_height = (computed.size().y * computed.inverse_scale_factor()).max(0.0);
    let max_scroll = workspace_sidebar_max_scroll(workspace_sidebar_rows(&state).len(), viewport_height);
    scroll_position.y = (scroll_position.y + delta_y).clamp(0.0, max_scroll);
    scroll_position.x = 0.0;
}

fn sync_workspace_selected_row_scroll(
    state: Res<EditorState>,
    mut last_selection: Local<Option<WorkspaceSelectedRow>>,
    mut list_query: Query<(&ComputedNode, &mut ScrollPosition), With<WorkspaceFileList>>,
) {
    if !state.workspace_sidebar_visible {
        *last_selection = state.workspace_selected_row.clone();
        return;
    }

    if *last_selection == state.workspace_selected_row {
        return;
    }
    *last_selection = state.workspace_selected_row.clone();

    let Some(_) = state.workspace_selected_row.as_ref() else {
        return;
    };
    let rows = workspace_sidebar_rows(&state);
    let Some(index) = selected_workspace_row_index(&state, &rows) else {
        return;
    };
    let Ok((computed, mut scroll_position)) = list_query.single_mut() else {
        return;
    };

    let viewport_height = (computed.size().y * computed.inverse_scale_factor()).max(0.0);
    if viewport_height <= f32::EPSILON {
        return;
    }

    let max_scroll = workspace_sidebar_max_scroll(rows.len(), viewport_height);
    let current_top = scroll_position.y.clamp(0.0, max_scroll);
    let current_bottom = current_top + viewport_height;
    let row_top = workspace_sidebar_row_top(index);
    let row_bottom = row_top + WORKSPACE_TREE_ROW_HEIGHT;

    let next_scroll = if row_top < current_top {
        row_top
    } else if row_bottom > current_bottom {
        row_bottom - viewport_height
    } else {
        current_top
    };

    scroll_position.y = next_scroll.clamp(0.0, max_scroll);
    scroll_position.x = 0.0;
}

fn folder_contains_active_file(
    active_relative_display: Option<&str>,
    folder_key: &str,
) -> bool {
    let Some(active_relative_display) = active_relative_display else {
        return false;
    };

    active_relative_display
        .strip_prefix(folder_key)
        .is_some_and(|suffix| suffix.starts_with('/'))
}

fn style_workspace_file_entry_text(
    state: Res<EditorState>,
    mut file_button_query: Query<
        (&Interaction, &WorkspaceFileButton, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_color_query: Query<&mut TextColor>,
) {
    for (interaction, workspace_file_button, children) in file_button_query.iter_mut() {
        let color = match *interaction {
            Interaction::Hovered | Interaction::Pressed => COLOR_WORKSPACE_FILE_HOVER,
            Interaction::None => {
                if state.workspace_active_file == Some(workspace_file_button.index) {
                    COLOR_WORKSPACE_FILE_SELECTED
                } else {
                    COLOR_WORKSPACE_FILE
                }
            }
        };

        for child in children.iter() {
            if let Ok(mut text_color) = text_color_query.get_mut(child) {
                text_color.0 = color;
            }
        }
    }
}
