const WORKSPACE_ROOT_LABEL_EMPTY: &str = "No workspace opened.";
const WORKSPACE_ROOT_LABEL_PREFIX: &str = "";
const WORKSPACE_EMPTY_RESULTS_LABEL: &str = "No .fountain/.md/.txt files found.";

// pretty sure these arent really working

// Horizontal gap between explorer left wall and the root label line.
const WORKSPACE_ROOT_LABEL_LEFT_MARGIN: f32 = 0.0;
// Horizontal gap between explorer left wall and folder/file tree rows.
const WORKSPACE_TREE_LIST_LEFT_MARGIN: f32 = 0.0;
const WORKSPACE_TREE_DEPTH_INDENT: f32 = 14.0;
const WORKSPACE_FILE_ROW_EXTRA_LEFT: f32 = 2.0;
const WORKSPACE_TREE_VERTICAL_PADDING: f32 = 10.0;

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
        self.clear_script_link_target_cache();

        match collect_workspace_files(&normalized_root) {
            Ok(files) => {
                self.workspace_files = files;
                self.workspace_expanded_folders =
                    default_expanded_workspace_folders(&self.workspace_files);
                self.sync_workspace_selection();
                self.status_message = format!(
                    "Opened workspace {} ({} files).",
                    normalized_root.display(),
                    self.workspace_files.len()
                );
            }
            Err(error) => {
                self.workspace_files.clear();
                self.workspace_selected = None;
                self.workspace_expanded_folders.clear();
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

    fn toggle_workspace_folder(&mut self, folder_key: &str) {
        if self.workspace_expanded_folders.contains(folder_key) {
            self.workspace_expanded_folders.remove(folder_key);
        } else {
            self.workspace_expanded_folders
                .insert(folder_key.to_owned());
        }
        self.workspace_ui_dirty = true;
    }

    fn sync_workspace_selection(&mut self) {
        self.workspace_selected = self
            .workspace_files
            .iter()
            .position(|entry| entry.path == self.paths.load_path);
        self.workspace_ui_dirty = true;
    }
}

fn workspace_sidebar_rows(state: &EditorState) -> Vec<WorkspaceSidebarRow> {
    let mut folders_by_parent = BTreeMap::<String, Vec<(String, String)>>::new();
    let mut files_by_parent = BTreeMap::<String, Vec<(usize, String)>>::new();

    for (index, file) in state.workspace_files.iter().enumerate() {
        let parent_key = workspace_parent_key(&file.relative_display);
        let file_name = workspace_base_name(&file.relative_display);
        files_by_parent
            .entry(parent_key)
            .or_default()
            .push((index, file_name));

        let components = file.relative_display.split('/').collect::<Vec<_>>();
        if components.len() <= 1 {
            continue;
        }

        let mut parent = String::new();
        for component in components.iter().take(components.len().saturating_sub(1)) {
            let folder_key = if parent.is_empty() {
                (*component).to_owned()
            } else {
                format!("{parent}/{component}")
            };

            let siblings = folders_by_parent.entry(parent.clone()).or_default();
            if !siblings
                .iter()
                .any(|(existing_key, _)| *existing_key == folder_key)
            {
                siblings.push((folder_key.clone(), (*component).to_owned()));
            }

            parent = folder_key;
        }
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

fn default_expanded_workspace_folders(files: &[WorkspaceFileEntry]) -> BTreeSet<String> {
    let mut expanded = BTreeSet::<String>::new();
    for file in files {
        let Some((top_level, _)) = file.relative_display.split_once('/') else {
            continue;
        };
        expanded.insert(top_level.to_owned());
    }
    expanded
}

fn collect_workspace_files(root: &Path) -> io::Result<Vec<WorkspaceFileEntry>> {
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

    files.sort_by(|left, right| left.relative_display.cmp(&right.relative_display));
    Ok(files)
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
        Some("fountain") | Some("txt") | Some("md") | Some("markdown")
    )
}

fn workspace_sidebar_bundle(font: Handle<Font>) -> impl Bundle {
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
        BackgroundColor(Color::srgb(0.86, 0.87, 0.89)),
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
                    row_gap: px(4.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
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
    let selected_relative_display = state
        .workspace_selected
        .and_then(|index| state.workspace_files.get(index))
        .map(|entry| entry.relative_display.as_str());

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
                    let folder_is_opened = expanded
                        || folder_contains_selected_file(selected_relative_display, &folder_key);
                    let folder_font = if folder_is_opened {
                        fonts.bold.clone()
                    } else {
                        fonts.regular.clone()
                    };

                    parent.spawn((
                        Button,
                        WorkspaceFolderToggleButton { folder_key },
                        Node {
                            width: percent(100.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(6.0),
                            padding: UiRect::axes(px(left_indent), px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
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
                    let text_color = if state.workspace_selected == Some(file_index) {
                        COLOR_WORKSPACE_FILE_SELECTED
                    } else {
                        COLOR_WORKSPACE_FILE
                    };

                    parent.spawn((
                        Button,
                        WorkspaceFileButton { index: file_index },
                        Node {
                            width: percent(100.0),
                            padding: UiRect::new(px(left_indent), px(8.0), px(4.0), px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
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

fn folder_contains_selected_file(
    selected_relative_display: Option<&str>,
    folder_key: &str,
) -> bool {
    let Some(selected_relative_display) = selected_relative_display else {
        return false;
    };

    selected_relative_display
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
                if state.workspace_selected == Some(workspace_file_button.index) {
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
