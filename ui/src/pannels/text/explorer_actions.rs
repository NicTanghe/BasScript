#[derive(Component)]
pub(crate) struct WorkspacePromptRoot;

#[derive(Component)]
pub(crate) struct WorkspacePromptTitle;

#[derive(Component)]
pub(crate) struct WorkspacePromptInput;

#[derive(Component)]
pub(crate) struct WorkspacePromptHint;

pub(crate) fn workspace_prompt_bundle(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            top: px(0.0),
            width: percent(100.0),
            height: percent(100.0),
            display: Display::None,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(24.0)),
            ..default()
        },
        BackgroundColor(COLOR_WORKSPACE_PROMPT_BACKDROP),
        ZIndex(100),
        GlobalZIndex(100),
        WorkspacePromptRoot,
        children![(
            Node {
                width: px(680.0),
                max_width: percent(94.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(10.0),
                padding: UiRect::all(px(14.0)),
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(px(7.0)),
                ..default()
            },
            BackgroundColor(COLOR_WORKSPACE_PROMPT_BG),
            BorderColor::all(Color::srgba(0.15, 0.17, 0.20, 0.22)),
            children![
                (
                    Text::new(""),
                    TextFont {
                        font: font.clone().into(),
                        font_size: FontSize::Px(14.0),
                        ..default()
                    },
                    TextColor(COLOR_TEXT_MAIN),
                    WorkspacePromptTitle,
                ),
                (
                    Node {
                        width: percent(100.0),
                        min_height: px(32.0),
                        padding: UiRect::axes(px(8.0), px(7.0)),
                        align_items: AlignItems::Center,
                        border: UiRect::all(px(1.0)),
                        border_radius: BorderRadius::all(px(5.0)),
                        ..default()
                    },
                    BackgroundColor(COLOR_WORKSPACE_PROMPT_INPUT_BG),
                    BorderColor::all(Color::srgba(0.15, 0.17, 0.20, 0.18)),
                    children![(
                        Text::new(""),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        TextColor(COLOR_TEXT_MAIN),
                        WorkspacePromptInput,
                    )],
                ),
                (
                    Text::new(""),
                    TextFont {
                        font: font.into(),
                        font_size: FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(COLOR_TEXT_MUTED),
                    WorkspacePromptHint,
                ),
            ],
        )],
    )
}

pub(crate) fn sync_workspace_prompt_ui(
    state: Res<EditorState>,
    mut root_query: Query<&mut Node, With<WorkspacePromptRoot>>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<WorkspacePromptTitle>>,
        Query<&mut Text, With<WorkspacePromptInput>>,
        Query<&mut Text, With<WorkspacePromptHint>>,
    )>,
) {
    let Some(prompt) = state.workspace_prompt.as_ref() else {
        if let Ok(mut root) = root_query.single_mut() {
            root.display = Display::None;
        }
        return;
    };

    if let Ok(mut root) = root_query.single_mut() {
        root.display = Display::Flex;
    }

    let (title, input, hint) = workspace_prompt_text(&state, prompt);
    if let Ok(mut title_text) = text_queries.p0().single_mut() {
        **title_text = title;
    }
    if let Ok(mut input_text) = text_queries.p1().single_mut() {
        **input_text = input;
    }
    if let Ok(mut hint_text) = text_queries.p2().single_mut() {
        **hint_text = hint;
    }
}

pub(crate) fn workspace_prompt_text(
    state: &EditorState,
    prompt: &WorkspacePrompt,
) -> (String, String, String) {
    match prompt {
        WorkspacePrompt::Create { input } => (
            "New file or folder".to_string(),
            format!("{input}_"),
            "Enter creates. A path ending in / or \\ creates a folder. Esc cancels.".to_string(),
        ),
        WorkspacePrompt::ChooseMarkdownTemplate {
            destination,
            templates,
            selected,
        } => {
            let options = std::iter::once("Blank".to_string())
                .chain(templates.iter().map(|template| template.name.clone()))
                .enumerate()
                .map(|(index, name)| {
                    if index == *selected {
                        format!("> {name}")
                    } else {
                        format!("  {name}")
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            let file_name = destination
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Markdown file");
            (
                format!("Choose a template for {file_name}"),
                options,
                "Up/Down or j/k selects. Enter creates. Esc cancels.".to_string(),
            )
        }
        WorkspacePrompt::Rename { target, input } => {
            let path = workspace_target_path(state, target);
            let target_kind = if path.as_ref().is_some_and(|path| path.is_dir()) {
                "folder"
            } else {
                "file"
            };
            (
                format!("Rename {target_kind}"),
                format!("{input}_"),
                "Enter renames. Edit the full destination path inside the workspace. Esc cancels."
                    .to_string(),
            )
        }
        WorkspacePrompt::Delete { target } => {
            let path = workspace_target_path(state, target);
            let label = path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "Unknown target".to_string());
            let non_empty = path
                .as_ref()
                .is_some_and(|path| path.is_dir() && workspace_dir_has_entries(path));
            let target_kind = if path.as_ref().is_some_and(|path| path.is_dir()) {
                if non_empty {
                    "non-empty folder"
                } else {
                    "folder"
                }
            } else {
                "file"
            };
            (
                format!("Delete {target_kind}?"),
                label,
                "Press Enter or y to delete the selected row. Press n or Esc to cancel."
                    .to_string(),
            )
        }
    }
}

pub(crate) fn handle_workspace_prompt_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
) {
    if state.workspace_prompt.is_none() {
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        state.workspace_prompt = None;
        state.status_message = "Explorer prompt canceled.".to_string();
        return;
    }

    if paste_shortcut_just_pressed(&keys) {
        let mut status_message = None::<String>;
        if let Some(prompt) = state.workspace_prompt.as_mut() {
            let input = match prompt {
                WorkspacePrompt::Create { input } | WorkspacePrompt::Rename { input, .. } => {
                    Some(input)
                }
                WorkspacePrompt::ChooseMarkdownTemplate { .. } | WorkspacePrompt::Delete { .. } => {
                    None
                }
            };
            if let Some(input) = input {
                if let Some(text) = read_system_clipboard_text() {
                    input.push_str(&text.replace('\n', ""));
                    status_message = Some("Pasted clipboard into explorer prompt.".to_string());
                } else {
                    status_message = Some("Clipboard is empty or unavailable.".to_string());
                }
            }
        }
        if let Some(message) = status_message {
            state.status_message = message;
        }
        for _ in keyboard_inputs.read() {}
        return;
    }

    let keybinds = state.keybinds.clone();
    let Some(prompt) = state.workspace_prompt.as_mut() else {
        return;
    };

    enum PromptAction {
        ConfirmCreate(String),
        ConfirmMarkdownTemplate(PathBuf, Option<WorkspaceMarkdownTemplate>),
        ConfirmRename(WorkspaceSelectedRow, String),
        ConfirmDelete(WorkspaceSelectedRow),
        Cancel,
    }

    let mut action = None::<PromptAction>;
    match prompt {
        WorkspacePrompt::Create { input } => {
            for key_input in keyboard_inputs.read() {
                if !key_input.state.is_pressed() {
                    continue;
                }

                if text_input_should_skip_for_shortcut(&keys, key_input, &keybinds) {
                    continue;
                }

                match &key_input.logical_key {
                    Key::Enter => {
                        action = Some(PromptAction::ConfirmCreate(input.clone()));
                        break;
                    }
                    Key::Backspace => {
                        input.pop();
                    }
                    Key::Delete => {}
                    _ => {
                        if let Some(inserted_text) = &key_input.text {
                            if !inserted_text.is_empty()
                                && inserted_text.chars().all(is_printable_char)
                            {
                                input.push_str(inserted_text);
                            }
                        }
                    }
                }
            }
        }
        WorkspacePrompt::ChooseMarkdownTemplate {
            destination,
            templates,
            selected,
        } => {
            let option_count = templates.len() + 1;
            if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyK) {
                *selected = if *selected == 0 {
                    option_count - 1
                } else {
                    *selected - 1
                };
            }
            if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyJ) {
                *selected = (*selected + 1) % option_count;
            }
            if keys.just_pressed(KeyCode::Enter) {
                let template = selected
                    .checked_sub(1)
                    .and_then(|index| templates.get(index))
                    .cloned();
                action = Some(PromptAction::ConfirmMarkdownTemplate(
                    destination.clone(),
                    template,
                ));
            }
            for _ in keyboard_inputs.read() {}
        }
        WorkspacePrompt::Rename { target, input } => {
            for key_input in keyboard_inputs.read() {
                if !key_input.state.is_pressed() {
                    continue;
                }

                if text_input_should_skip_for_shortcut(&keys, key_input, &keybinds) {
                    continue;
                }

                match &key_input.logical_key {
                    Key::Enter => {
                        action = Some(PromptAction::ConfirmRename(target.clone(), input.clone()));
                        break;
                    }
                    Key::Backspace => {
                        input.pop();
                    }
                    Key::Delete => {}
                    _ => {
                        if let Some(inserted_text) = &key_input.text {
                            if !inserted_text.is_empty()
                                && inserted_text.chars().all(is_printable_char)
                            {
                                input.push_str(inserted_text);
                            }
                        }
                    }
                }
            }
        }
        WorkspacePrompt::Delete { target } => {
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyY) {
                action = Some(PromptAction::ConfirmDelete(target.clone()));
            } else if keys.just_pressed(KeyCode::KeyN) {
                action = Some(PromptAction::Cancel);
            }
        }
    }

    match action {
        Some(PromptAction::ConfirmCreate(input)) => {
            state.workspace_prompt = None;
            confirm_workspace_create(&mut state, &input);
        }
        Some(PromptAction::ConfirmMarkdownTemplate(destination, template)) => {
            state.workspace_prompt = None;
            confirm_workspace_markdown_template(&mut state, destination, template.as_ref());
        }
        Some(PromptAction::ConfirmRename(target, input)) => {
            state.workspace_prompt = None;
            confirm_workspace_rename(&mut state, target, &input);
        }
        Some(PromptAction::ConfirmDelete(target)) => {
            state.workspace_prompt = None;
            confirm_workspace_delete(&mut state, target);
        }
        Some(PromptAction::Cancel) => {
            state.workspace_prompt = None;
            state.status_message = "Explorer prompt canceled.".to_string();
        }
        None => {}
    }
}

pub(crate) fn handle_workspace_keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut repeat: ResMut<WorkspaceSelectionRepeatState>,
    mut state: ResMut<EditorState>,
) {
    if !state.workspace_focused
        || state.workspace_prompt.is_some()
        || state.command_menu.is_some()
        || shortcut_modifier_pressed(&keys)
    {
        repeat.active_arrow = None;
        repeat.repeat_cooldown_secs = 0.0;
        return;
    }

    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyQ) {
        state.workspace_focused = false;
        state.workspace_ui_dirty = true;
        state.status_message = "Explorer focus left.".to_string();
        return;
    }

    handle_workspace_selection_repeat(&keys, &time, &mut repeat, &mut state);
    if keys.just_pressed(KeyCode::KeyH) {
        collapse_or_select_workspace_parent(&mut state);
    }
    if keys.just_pressed(KeyCode::KeyL) {
        expand_or_open_workspace_selection(&mut state);
    }
    if keys.just_pressed(KeyCode::KeyO) || keys.just_pressed(KeyCode::Enter) {
        open_or_toggle_workspace_selection(&mut state);
    }
    if keys.just_pressed(KeyCode::KeyN) {
        begin_workspace_create_prompt(&mut state);
    }
    if keys.just_pressed(KeyCode::KeyD) {
        begin_workspace_delete_prompt(&mut state);
    }
    if keys.just_pressed(KeyCode::KeyR) {
        if shift_modifier_pressed(&keys) {
            state.refresh_workspace();
            state.status_message = "Workspace refreshed.".to_string();
        } else {
            begin_workspace_rename_prompt(&mut state);
        }
    }
}

pub(crate) fn handle_workspace_selection_repeat(
    keys: &ButtonInput<KeyCode>,
    time: &Time,
    repeat: &mut WorkspaceSelectionRepeatState,
    state: &mut EditorState,
) {
    let previous_active_arrow = repeat.active_arrow;
    for key in [
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::KeyK,
        KeyCode::KeyJ,
    ] {
        if keys.just_pressed(key) {
            move_workspace_selection(state, workspace_selection_key_delta(key));
            repeat.active_arrow = Some(key);
            repeat.repeat_cooldown_secs = WORKSPACE_SELECTION_REPEAT_INITIAL_DELAY_SECS;
            return;
        }
    }

    let active_arrow = repeat
        .active_arrow
        .filter(|arrow| keys.pressed(*arrow))
        .or_else(|| held_workspace_selection_key(keys));

    if active_arrow != previous_active_arrow {
        repeat.repeat_cooldown_secs = WORKSPACE_SELECTION_REPEAT_INITIAL_DELAY_SECS;
    }

    repeat.active_arrow = active_arrow;
    let Some(arrow) = active_arrow else {
        repeat.repeat_cooldown_secs = 0.0;
        return;
    };

    repeat.repeat_cooldown_secs -= time.delta_secs().max(0.0);
    while repeat.repeat_cooldown_secs <= 0.0 {
        move_workspace_selection(state, workspace_selection_key_delta(arrow));
        repeat.repeat_cooldown_secs += WORKSPACE_SELECTION_REPEAT_INTERVAL_SECS;
    }
}

pub(crate) fn held_workspace_selection_key(keys: &ButtonInput<KeyCode>) -> Option<KeyCode> {
    [
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::KeyK,
        KeyCode::KeyJ,
    ]
    .into_iter()
    .find(|key| keys.pressed(*key))
}

pub(crate) fn workspace_selection_key_delta(key: KeyCode) -> isize {
    match key {
        KeyCode::ArrowUp | KeyCode::KeyK => -1,
        _ => 1,
    }
}

pub(crate) fn move_workspace_selection(state: &mut EditorState, delta: isize) {
    let rows = workspace_sidebar_rows(state);
    if rows.is_empty() {
        state.status_message = "Explorer has no rows to select.".to_string();
        return;
    }

    let current = selected_workspace_row_index(state, &rows);
    let next = match (current, delta.is_negative()) {
        (Some(index), true) => index.saturating_sub(delta.unsigned_abs()),
        (Some(index), false) => index.saturating_add(delta as usize).min(rows.len() - 1),
        (None, true) => rows.len() - 1,
        (None, false) => 0,
    };

    select_workspace_row_at_index(state, &rows, next);
}

pub(crate) fn collapse_or_select_workspace_parent(state: &mut EditorState) {
    let Some(selection) = state.workspace_selected_row.clone() else {
        state.status_message = "No explorer row selected.".to_string();
        return;
    };

    match selection {
        WorkspaceSelectedRow::Folder(folder_key) => {
            if state.workspace_expanded_folders.remove(&folder_key) {
                state.workspace_ui_dirty = true;
                return;
            }
            state.workspace_selected_row = workspace_parent_folder_selection(&folder_key);
            state.workspace_ui_dirty = true;
        }
        WorkspaceSelectedRow::File(path) => {
            state.workspace_selected_row = workspace_parent_selection_for_file(state, &path);
            state.workspace_ui_dirty = true;
        }
    }
}

pub(crate) fn expand_or_open_workspace_selection(state: &mut EditorState) {
    let Some(selection) = state.workspace_selected_row.clone() else {
        state.status_message = "No explorer row selected.".to_string();
        return;
    };

    match selection {
        WorkspaceSelectedRow::Folder(folder_key) => {
            state.workspace_expanded_folders.insert(folder_key);
            state.workspace_ui_dirty = true;
        }
        WorkspaceSelectedRow::File(path) => {
            open_workspace_path_from_selection(state, &path);
        }
    }
}

pub(crate) fn open_or_toggle_workspace_selection(state: &mut EditorState) {
    let Some(selection) = state.workspace_selected_row.clone() else {
        state.status_message = "No explorer row selected.".to_string();
        return;
    };

    match selection {
        WorkspaceSelectedRow::Folder(folder_key) => state.toggle_workspace_folder(&folder_key),
        WorkspaceSelectedRow::File(path) => open_workspace_path_from_selection(state, &path),
    }
}

pub(crate) fn open_workspace_path_from_selection(state: &mut EditorState, path: &Path) {
    if let Some(index) = state
        .workspace_files
        .iter()
        .position(|entry| workspace_paths_match(&entry.path, path))
    {
        state.open_workspace_file(index);
    } else {
        state.navigate_to_path(path.to_path_buf());
    }
}

pub(crate) fn begin_workspace_create_prompt(state: &mut EditorState) {
    let Some(folder) = selected_workspace_folder_context(state) else {
        state.status_message = "Open a workspace before creating files.".to_string();
        return;
    };

    state.workspace_focused = true;
    state.close_link_autocomplete();
    state.workspace_prompt = Some(WorkspacePrompt::Create {
        input: path_with_trailing_separator(&folder),
    });
}

pub(crate) fn begin_workspace_delete_prompt(state: &mut EditorState) {
    let Some(target) = state.workspace_selected_row.clone() else {
        state.status_message = "Select a file or folder to delete.".to_string();
        return;
    };
    let Some(path) = workspace_target_path(state, &target) else {
        state.status_message = "Explorer selection has no filesystem target.".to_string();
        return;
    };
    if !path.exists() {
        state.status_message = format!("Delete target does not exist: {}", path.display());
        state.refresh_workspace();
        return;
    }

    state.workspace_focused = true;
    state.close_link_autocomplete();
    state.workspace_prompt = Some(WorkspacePrompt::Delete { target });
}

pub(crate) fn begin_workspace_rename_prompt(state: &mut EditorState) {
    let Some(target) = state.workspace_selected_row.clone() else {
        state.status_message = "Select a file or folder to rename.".to_string();
        return;
    };
    let Some(path) = workspace_target_path(state, &target) else {
        state.status_message = "Explorer selection has no filesystem target.".to_string();
        return;
    };
    if !path.exists() {
        state.status_message = format!("Rename target does not exist: {}", path.display());
        state.refresh_workspace();
        return;
    }

    state.workspace_focused = true;
    state.close_link_autocomplete();
    state.workspace_prompt = Some(WorkspacePrompt::Rename {
        target,
        input: path.display().to_string(),
    });
}

pub(crate) fn confirm_workspace_create(state: &mut EditorState, input: &str) {
    let raw = input.trim();
    let Some(root) = state.workspace_root.clone() else {
        state.status_message = "Open a workspace before creating files.".to_string();
        return;
    };
    let Some(folder_context) = selected_workspace_folder_context(state) else {
        state.status_message = "No folder selected for create.".to_string();
        return;
    };

    let empty_candidate = PathBuf::from(raw.trim_end_matches(|ch| ch == '/' || ch == '\\'));
    if raw.is_empty() || workspace_paths_match(&empty_candidate, &folder_context) {
        state.status_message = "Create path is empty.".to_string();
        return;
    }

    let is_folder = raw.ends_with('/') || raw.ends_with('\\');
    let path = PathBuf::from(raw);
    if !path.is_absolute() {
        state.status_message = "Create path must be absolute.".to_string();
        return;
    }
    if !workspace_path_is_under_root(&root, &path) {
        state.status_message = "Create path must stay inside the workspace.".to_string();
        return;
    }

    if is_folder {
        match fs::create_dir_all(&path) {
            Ok(()) => {
                state.refresh_workspace();
                if let Some(folder_key) = workspace_folder_key_for_path(&root, &path) {
                    expand_workspace_parent_folders(state, &folder_key);
                    state.workspace_selected_row = Some(WorkspaceSelectedRow::Folder(folder_key));
                }
                state.workspace_ui_dirty = true;
                state.status_message = format!("Created folder {}", path.display());
            }
            Err(error) => {
                state.status_message =
                    format!("Create folder failed for {}: {error}", path.display());
            }
        }
        return;
    }

    if is_markdown_file_path(&path) {
        let templates = collect_workspace_markdown_templates().unwrap_or_else(|error| {
            warn!("[explorer] Failed reading Markdown templates: {error}");
            Vec::new()
        });
        state.workspace_prompt = Some(WorkspacePrompt::ChooseMarkdownTemplate {
            destination: path,
            templates,
            selected: 0,
        });
        state.status_message = "Choose a template for the new Markdown file.".to_string();
        return;
    }

    create_workspace_file(state, path, "", None);
}

pub(crate) fn confirm_workspace_markdown_template(
    state: &mut EditorState,
    destination: PathBuf,
    template: Option<&WorkspaceMarkdownTemplate>,
) {
    let (contents, template_name) = match template {
        Some(template) => match fs::read_to_string(&template.path) {
            Ok(contents) => (
                personalize_workspace_markdown_template(
                    &contents,
                    &destination,
                    template,
                    &chrono::Local::now().format("%Y-%m-%d").to_string(),
                ),
                Some(template.name.as_str()),
            ),
            Err(error) => {
                state.status_message = format!(
                    "Could not read Markdown template {}: {error}",
                    template.path.display()
                );
                return;
            }
        },
        None => (String::new(), None),
    };

    create_workspace_file(state, destination, &contents, template_name);
}

pub(crate) fn create_workspace_file(
    state: &mut EditorState,
    path: PathBuf,
    contents: &str,
    template_name: Option<&str>,
) {
    let Some(root) = state.workspace_root.clone() else {
        state.status_message = "Open a workspace before creating files.".to_string();
        return;
    };

    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            state.status_message = format!(
                "Create parent folders failed for {}: {error}",
                parent.display()
            );
            return;
        }
    }

    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut file) => {
            use std::io::Write;

            if let Err(error) = file.write_all(contents.as_bytes()) {
                drop(file);
                let _ = fs::remove_file(&path);
                state.status_message =
                    format!("Create file failed for {}: {error}", path.display());
                return;
            }
            let _ = state.load_from_path(path.clone());
            state.refresh_workspace();
            if let Some(parent_key) = path
                .parent()
                .and_then(|parent| workspace_folder_key_for_path(&root, parent))
            {
                expand_workspace_parent_folders(state, &parent_key);
            }
            state.workspace_selected_row = Some(WorkspaceSelectedRow::File(path.clone()));
            state.workspace_focused = true;
            state.workspace_ui_dirty = true;
            state.status_message = template_name.map_or_else(
                || format!("Created file {}", path.display()),
                |name| format!("Created file {} from {name} template", path.display()),
            );
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            state.status_message = format!("File already exists: {}", path.display());
        }
        Err(error) => {
            state.status_message = format!("Create file failed for {}: {error}", path.display());
        }
    }
}

pub(crate) fn is_markdown_file_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
        })
}

pub(crate) fn collect_workspace_markdown_templates() -> io::Result<Vec<WorkspaceMarkdownTemplate>> {
    let mut templates = Vec::new();
    for entry in fs::read_dir("assets/templates")? {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_file() || !is_markdown_file_path(&path) {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        templates.push(WorkspaceMarkdownTemplate {
            name: workspace_template_display_name(stem),
            path,
        });
    }
    templates.sort_by_key(|template| template.name.to_ascii_lowercase());
    Ok(templates)
}

pub(crate) fn workspace_template_display_name(stem: &str) -> String {
    let words = stem
        .strip_suffix("-template")
        .or_else(|| stem.strip_suffix("_template"))
        .unwrap_or(stem)
        .split(['-', '_'])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first
                    .to_uppercase()
                    .chain(chars.as_str().to_lowercase().chars())
                    .collect::<String>()
            })
        })
        .collect::<Vec<_>>();
    if words.is_empty() {
        "Unnamed".to_string()
    } else {
        words.join(" ")
    }
}

pub(crate) fn personalize_workspace_markdown_template(
    contents: &str,
    destination: &Path,
    template: &WorkspaceMarkdownTemplate,
    date: &str,
) -> String {
    let filename_stem = destination
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("untitled");
    let display_name = workspace_filename_display_name(filename_stem);
    let target = workspace_filename_target(filename_stem);
    let entity_type = workspace_filename_target(&template.name);
    let id = format!(
        "entity_{}_{}_001",
        entity_type.replace('-', "_"),
        target.replace('-', "_")
    );
    let aliases = format!("[{}]", yaml_scalar(filename_stem));

    let mut in_front_matter = false;
    let mut front_matter_finished = false;
    let mut personalized = String::with_capacity(contents.len() + display_name.len());
    for raw_line in contents.split_inclusive('\n') {
        let (line, newline) = raw_line
            .strip_suffix('\n')
            .map_or((raw_line, ""), |line| (line, "\n"));
        let trimmed = line.trim().trim_start_matches('\u{feff}');
        if !front_matter_finished && trimmed == "---" {
            if in_front_matter {
                front_matter_finished = true;
            } else {
                in_front_matter = true;
            }
            personalized.push_str(line);
            personalized.push_str(newline);
            continue;
        }

        if in_front_matter && !front_matter_finished {
            let indent_length = line.len() - line.trim_start().len();
            let indent = &line[..indent_length];
            let replacement = line.trim_start().split_once(':').and_then(|(key, _)| {
                let value = match key.trim() {
                    "id" => id.clone(),
                    "target" => target.clone(),
                    "type" => entity_type.clone(),
                    "name" => yaml_scalar(&display_name),
                    "aliases" => aliases.clone(),
                    _ => return None,
                };
                Some(format!("{indent}{}: {value}", key.trim()))
            });
            personalized.push_str(replacement.as_deref().unwrap_or(line));
        } else {
            personalized.push_str(line);
        }
        personalized.push_str(newline);
    }

    personalized
        .replace("# Character name unlinked", &format!("# {display_name}"))
        .replace("# Title", &format!("# {display_name}"))
        .replace("*type*", &format!("*{entity_type}*"))
        .replace("DATE", date)
}

pub(crate) fn workspace_filename_display_name(stem: &str) -> String {
    let words = stem
        .split(|ch: char| ch == '_' || ch == '-' || ch.is_whitespace())
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first
                    .to_uppercase()
                    .chain(chars.as_str().to_lowercase().chars())
                    .collect::<String>()
            })
        })
        .collect::<Vec<_>>();
    if words.is_empty() {
        "Untitled".to_string()
    } else {
        words.join(" ")
    }
}

pub(crate) fn workspace_filename_target(stem: &str) -> String {
    let mut target = String::new();
    let mut previous_was_separator = true;
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() {
            target.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator {
            target.push('-');
            previous_was_separator = true;
        }
    }
    while target.ends_with('-') {
        target.pop();
    }
    if target.is_empty() {
        "untitled".to_string()
    } else {
        target
    }
}

#[cfg(test)]
mod workspace_markdown_template_tests {
    use super::*;

    #[test]
    fn recognizes_markdown_extensions_case_insensitively() {
        assert!(is_markdown_file_path(Path::new("character.MD")));
        assert!(is_markdown_file_path(Path::new("plot.markdown")));
        assert!(!is_markdown_file_path(Path::new("script.fountain")));
    }

    #[test]
    fn turns_template_file_stems_into_labels() {
        assert_eq!(
            workspace_template_display_name("character-template"),
            "Character"
        );
        assert_eq!(
            workspace_template_display_name("minor_plot_template"),
            "Minor Plot"
        );
    }

    #[test]
    fn guesses_document_fields_from_the_filename() {
        assert_eq!(
            workspace_filename_display_name("queen_quesha"),
            "Queen Quesha"
        );
        assert_eq!(workspace_filename_target("queen_quesha"), "queen-quesha");
    }

    #[test]
    fn personalizes_front_matter_heading_type_and_date() {
        let template = WorkspaceMarkdownTemplate {
            name: "Character".to_string(),
            path: PathBuf::from("assets/templates/character-template.md"),
        };
        let source = "\u{feff}---\nid: entity_character_template_001\ntarget: character-template\ntype: template\nname: 'Character Template'\naliases: [Character_template]\nstatus: draft\n---\n# Character name unlinked\n*type*\nCreated DATE\n";

        let personalized = personalize_workspace_markdown_template(
            source,
            Path::new("/project/queen_quesha.md"),
            &template,
            "2026-07-12",
        );

        assert!(personalized.contains("id: entity_character_queen_quesha_001"));
        assert!(personalized.contains("target: queen-quesha"));
        assert!(personalized.contains("type: character"));
        assert!(personalized.contains("name: 'Queen Quesha'"));
        assert!(personalized.contains("aliases: [queen_quesha]"));
        assert!(personalized.contains("# Queen Quesha"));
        assert!(personalized.contains("*character*"));
        assert!(personalized.contains("Created 2026-07-12"));
    }
}

pub(crate) fn confirm_workspace_rename(
    state: &mut EditorState,
    target: WorkspaceSelectedRow,
    input: &str,
) {
    let Some(root) = state.workspace_root.clone() else {
        state.status_message = "Open a workspace before renaming files.".to_string();
        return;
    };
    let Some(source) = workspace_target_path(state, &target) else {
        state.status_message = "Explorer selection has no filesystem target.".to_string();
        return;
    };
    if !source.exists() {
        state.status_message = format!("Rename target does not exist: {}", source.display());
        state.refresh_workspace();
        return;
    }
    if !workspace_path_is_under_root(&root, &source) {
        state.status_message = "Rename target must be inside the workspace.".to_string();
        return;
    }

    let destination = match workspace_rename_destination(&source, input) {
        Ok(destination) => destination,
        Err(message) => {
            state.status_message = message;
            return;
        }
    };
    if workspace_paths_match(&source, &destination) {
        state.status_message = "Rename path is unchanged.".to_string();
        return;
    }
    if !workspace_destination_is_under_root(&root, &destination) {
        state.status_message = "Rename path must stay inside the workspace.".to_string();
        return;
    }
    if destination.exists() {
        state.status_message = format!("Rename target already exists: {}", destination.display());
        return;
    }

    let source_is_dir = source.is_dir();
    if source_is_dir
        && destination
            .parent()
            .is_some_and(|parent| workspace_path_is_under_root(&source, parent))
    {
        state.status_message = "Cannot rename a folder into itself.".to_string();
        return;
    }

    match fs::rename(&source, &destination) {
        Ok(()) => {
            let active_rebased =
                rebase_workspace_active_paths_after_rename(state, &source, &destination);
            if active_rebased {
                state.document_format =
                    detect_document_format(&state.paths.load_path, &state.document);
                state.set_zoom(state.zoom);
                state.clear_script_link_target_cache();
                state.reparse();
            }

            state.refresh_workspace();
            if source_is_dir {
                if let Some(folder_key) = workspace_folder_key_for_path(&root, &destination) {
                    expand_workspace_parent_folders(state, &folder_key);
                    state.workspace_expanded_folders.insert(folder_key.clone());
                    state.workspace_selected_row = Some(WorkspaceSelectedRow::Folder(folder_key));
                }
            } else {
                if let Some(parent_key) = destination
                    .parent()
                    .and_then(|parent| workspace_folder_key_for_path(&root, parent))
                {
                    expand_workspace_parent_folders(state, &parent_key);
                }
                state.workspace_selected_row =
                    Some(WorkspaceSelectedRow::File(destination.clone()));
            }
            state.workspace_focused = true;
            state.workspace_ui_dirty = true;
            state.status_message =
                format!("Renamed {} to {}", source.display(), destination.display());
        }
        Err(error) => {
            state.status_message = format!(
                "Rename failed for {} to {}: {error}",
                source.display(),
                destination.display()
            );
        }
    }
}

pub(crate) fn confirm_workspace_delete(state: &mut EditorState, target: WorkspaceSelectedRow) {
    let Some(path) = workspace_target_path(state, &target) else {
        state.status_message = "Explorer selection has no filesystem target.".to_string();
        return;
    };
    let rows = workspace_sidebar_rows(state);
    let deleted_index = selected_workspace_row_index(state, &rows).unwrap_or(0);
    let active_deleted = workspace_active_path_affected(state, &path);

    let result = if path.is_dir() {
        fs::remove_dir_all(&path)
    } else {
        fs::remove_file(&path)
    };

    match result {
        Ok(()) => {
            if active_deleted {
                state.clear_deleted_active_document();
            }
            state.refresh_workspace();
            select_near_workspace_row_index(state, deleted_index);
            state.workspace_focused = true;
            state.workspace_ui_dirty = true;
            state.status_message = format!("Deleted {}", path.display());
        }
        Err(error) => {
            state.status_message = format!("Delete failed for {}: {error}", path.display());
        }
    }
}

pub(crate) fn selected_workspace_folder_context(state: &EditorState) -> Option<PathBuf> {
    let root = state.workspace_root.as_ref()?;
    match state.workspace_selected_row.as_ref() {
        Some(WorkspaceSelectedRow::Folder(folder_key)) => {
            Some(workspace_path_for_folder_key(root, folder_key))
        }
        Some(WorkspaceSelectedRow::File(path)) => path.parent().map(Path::to_path_buf),
        None => Some(root.clone()),
    }
}

pub(crate) fn workspace_selection_for_row(
    state: &EditorState,
    row: &WorkspaceSidebarRow,
) -> Option<WorkspaceSelectedRow> {
    match row {
        WorkspaceSidebarRow::Folder { folder_key, .. } => {
            Some(WorkspaceSelectedRow::Folder(folder_key.clone()))
        }
        WorkspaceSidebarRow::File { file_index, .. } => state
            .workspace_files
            .get(*file_index)
            .map(|entry| WorkspaceSelectedRow::File(entry.path.clone())),
    }
}

pub(crate) fn workspace_row_matches_selection(
    state: &EditorState,
    row: &WorkspaceSidebarRow,
    selection: &WorkspaceSelectedRow,
) -> bool {
    match (row, selection) {
        (
            WorkspaceSidebarRow::Folder { folder_key, .. },
            WorkspaceSelectedRow::Folder(selected_key),
        ) => folder_key == selected_key,
        (WorkspaceSidebarRow::File { file_index, .. }, WorkspaceSelectedRow::File(path)) => state
            .workspace_files
            .get(*file_index)
            .is_some_and(|entry| workspace_paths_match(&entry.path, path)),
        _ => false,
    }
}

pub(crate) fn selected_workspace_row_index(
    state: &EditorState,
    rows: &[WorkspaceSidebarRow],
) -> Option<usize> {
    let selection = state.workspace_selected_row.as_ref()?;
    rows.iter()
        .position(|row| workspace_row_matches_selection(state, row, selection))
}

pub(crate) fn select_workspace_row_at_index(
    state: &mut EditorState,
    rows: &[WorkspaceSidebarRow],
    index: usize,
) {
    if let Some(selection) = rows
        .get(index)
        .and_then(|row| workspace_selection_for_row(state, row))
    {
        state.workspace_selected_row = Some(selection);
        state.workspace_focused = true;
        state.workspace_ui_dirty = true;
    }
}

pub(crate) fn select_near_workspace_row_index(state: &mut EditorState, preferred_index: usize) {
    let rows = workspace_sidebar_rows(state);
    if rows.is_empty() {
        state.workspace_selected_row = None;
        return;
    }
    let index = preferred_index.min(rows.len() - 1);
    select_workspace_row_at_index(state, &rows, index);
}

pub(crate) fn workspace_parent_folder_selection(folder_key: &str) -> Option<WorkspaceSelectedRow> {
    workspace_parent_key(folder_key)
        .is_empty()
        .then_some(None)
        .unwrap_or_else(|| {
            Some(WorkspaceSelectedRow::Folder(workspace_parent_key(
                folder_key,
            )))
        })
}

pub(crate) fn workspace_parent_selection_for_file(
    state: &EditorState,
    path: &Path,
) -> Option<WorkspaceSelectedRow> {
    let root = state.workspace_root.as_ref()?;
    let parent = path.parent()?;
    workspace_folder_key_for_path(root, parent).map(WorkspaceSelectedRow::Folder)
}

pub(crate) fn workspace_target_path(
    state: &EditorState,
    target: &WorkspaceSelectedRow,
) -> Option<PathBuf> {
    let root = state.workspace_root.as_ref()?;
    match target {
        WorkspaceSelectedRow::Folder(folder_key) => {
            Some(workspace_path_for_folder_key(root, folder_key))
        }
        WorkspaceSelectedRow::File(path) => Some(path.clone()),
    }
}

pub(crate) fn workspace_path_for_folder_key(root: &Path, folder_key: &str) -> PathBuf {
    folder_key
        .split('/')
        .filter(|part| !part.is_empty())
        .fold(root.to_path_buf(), |path, part| path.join(part))
}

pub(crate) fn workspace_folder_key_for_path(root: &Path, path: &Path) -> Option<String> {
    let relative = path
        .strip_prefix(root)
        .ok()?
        .to_string_lossy()
        .replace('\\', "/");
    let key = relative.trim_matches('/').to_string();
    if key.is_empty() { None } else { Some(key) }
}

pub(crate) fn expand_workspace_parent_folders(state: &mut EditorState, folder_key: &str) {
    let mut current = String::new();
    for part in folder_key.split('/').filter(|part| !part.is_empty()) {
        current = if current.is_empty() {
            part.to_string()
        } else {
            format!("{current}/{part}")
        };
        state.workspace_expanded_folders.insert(current.clone());
    }
}

pub(crate) fn path_with_trailing_separator(path: &Path) -> String {
    let mut text = path.display().to_string();
    if !text.ends_with('/') && !text.ends_with('\\') {
        text.push(std::path::MAIN_SEPARATOR);
    }
    text
}

pub(crate) fn workspace_rename_destination(source: &Path, input: &str) -> Result<PathBuf, String> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err("Rename path is empty.".to_string());
    }

    let candidate = PathBuf::from(raw);
    let destination = if candidate.is_absolute() {
        candidate
    } else {
        source
            .parent()
            .ok_or_else(|| "Rename target has no parent folder.".to_string())?
            .join(candidate)
    };

    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Rename path must include a file or folder name.".to_string())?;
    if file_name == "." || file_name == ".." {
        return Err("Rename path must include a valid file or folder name.".to_string());
    }

    let parent = destination
        .parent()
        .ok_or_else(|| "Rename path has no parent folder.".to_string())?;
    if !parent.is_dir() {
        return Err(format!(
            "Rename parent folder does not exist: {}",
            parent.display()
        ));
    }

    Ok(parent
        .canonicalize()
        .unwrap_or_else(|_| parent.to_path_buf())
        .join(file_name))
}

pub(crate) fn workspace_destination_is_under_root(root: &Path, destination: &Path) -> bool {
    let Some(parent) = destination.parent() else {
        return false;
    };
    if !parent.is_dir() {
        return false;
    }
    workspace_path_is_under_root(root, parent)
}

pub(crate) fn workspace_path_is_under_root(root: &Path, path: &Path) -> bool {
    let root_key = workspace_path_key(root);
    let path_key = workspace_path_key(path);
    path_key == root_key || path_key.starts_with(&format!("{root_key}/"))
}

pub(crate) fn workspace_paths_match(left: &Path, right: &Path) -> bool {
    workspace_path_key(left) == workspace_path_key(right)
}

pub(crate) fn workspace_path_key(path: &Path) -> String {
    let normalized = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");
    if cfg!(windows) {
        normalized.to_ascii_lowercase()
    } else {
        normalized
    }
}

pub(crate) fn workspace_dir_has_entries(path: &Path) -> bool {
    fs::read_dir(path)
        .ok()
        .and_then(|mut entries| entries.next())
        .is_some()
}

pub(crate) fn workspace_active_path_affected(state: &EditorState, deleted_path: &Path) -> bool {
    let active_key = workspace_path_key(&state.paths.load_path);
    let deleted_key = workspace_path_key(deleted_path);
    active_key == deleted_key || active_key.starts_with(&format!("{deleted_key}/"))
}

pub(crate) fn rebase_workspace_active_paths_after_rename(
    state: &mut EditorState,
    source: &Path,
    destination: &Path,
) -> bool {
    let mut changed = false;
    if let Some(path) = workspace_rebase_path(&state.paths.load_path, source, destination) {
        state.paths.load_path = path;
        changed = true;
    }
    if let Some(path) = workspace_rebase_path(&state.paths.save_path, source, destination) {
        state.paths.save_path = path;
        changed = true;
    }
    changed
}

pub(crate) fn workspace_rebase_path(
    path: &Path,
    source: &Path,
    destination: &Path,
) -> Option<PathBuf> {
    if workspace_paths_match(path, source) {
        return Some(destination.to_path_buf());
    }

    let path_key = workspace_path_key(path);
    let source_key = workspace_path_key(source);
    let prefix = format!("{source_key}/");
    let suffix = path_key.strip_prefix(&prefix)?;
    Some(
        suffix
            .split('/')
            .filter(|part| !part.is_empty())
            .fold(destination.to_path_buf(), |path, part| path.join(part)),
    )
}

impl EditorState {
    pub(crate) fn clear_deleted_active_document(&mut self) {
        self.document = Document::new();
        self.document_format = DocumentFormat::Fountain;
        self.parsed = parse_document_with_format(&self.document, self.document_format);
        self.cursor = Cursor::default();
        self.selection_anchor = None;
        self.top_line = 0;
        self.processed_top_line = 0;
        self.processed_top_visual = 0;
        self.plain_horizontal_scroll = 0.0;
        self.processed_horizontal_scroll = 0.0;
        self.processed_zoom_anchor_bias_px = 0.0;
        self.processed_cache = None;
        self.processed_cache_dirty_from_line = Some(0);
        self.clear_history();

        let fallback = self
            .workspace_root
            .as_ref()
            .map(|root| root.join("untitled.fountain"))
            .unwrap_or_else(|| PathBuf::from(DEFAULT_SAVE_PATH));
        self.paths.load_path = fallback.clone();
        self.paths.save_path = fallback;
        self.workspace_active_file = None;
    }
}
#[allow(unused_imports)]
use super::*;
