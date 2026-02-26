fn shortcut_modifier_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ])
}

fn shift_modifier_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
}

fn shortcut_just_pressed(keys: &ButtonInput<KeyCode>, binding: ShortcutBinding) -> bool {
    if !shortcut_modifier_pressed(keys) {
        return false;
    }

    let shift_pressed = shift_modifier_pressed(keys);
    if binding.shift && !shift_pressed {
        return false;
    }
    if !binding.shift && shift_pressed {
        return false;
    }

    keys.just_pressed(binding.key)
}

fn handle_window_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
) {
    let binding = state.keybinds.binding(ShortcutAction::ToggleTopMenu);
    if !shortcut_just_pressed(&keys, binding) {
        return;
    }

    state.top_menu_collapsed = !state.top_menu_collapsed;
    state.status_message = format!(
        "Top menu: {}",
        if state.top_menu_collapsed {
            "HIDDEN"
        } else {
            "VISIBLE"
        }
    );
    info!(
        "[ui] Top-menu shortcut toggled top menu to {}",
        if state.top_menu_collapsed {
            "HIDDEN"
        } else {
            "VISIBLE"
        }
    );
}

fn sync_window_chrome(
    state: Res<EditorState>,
    mut primary_window_query: Query<(Entity, &mut Window), With<PrimaryWindow>>,
) {
    let Ok((window_entity, mut primary_window)) = primary_window_query.single_mut() else {
        return;
    };

    let state_changed = state.is_changed();
    let show_system_titlebar = state.show_system_titlebar;
    let decorations_changed = primary_window.decorations != show_system_titlebar;
    if decorations_changed {
        primary_window.decorations = show_system_titlebar;
    }

    #[cfg(target_os = "windows")]
    if decorations_changed || state_changed {
        apply_windows_chrome_preferences(window_entity, show_system_titlebar);
    }
}

#[cfg(target_os = "windows")]
fn apply_windows_chrome_preferences(window_entity: Entity, show_system_titlebar: bool) {
    use bevy::winit::WINIT_WINDOWS;
    use winit::platform::windows::{CornerPreference, WindowExtWindows};

    WINIT_WINDOWS.with_borrow(|winit_windows| {
        let Some(window) = winit_windows.get_window(window_entity) else {
            return;
        };

        window.set_corner_preference(CornerPreference::Round);
        window.set_undecorated_shadow(!show_system_titlebar);
    });
}

fn handle_file_shortcuts(
    _dialog_main_thread: NonSend<DialogMainThreadMarker>,
    keys: Res<ButtonInput<KeyCode>>,
    primary_window_query: Query<&RawHandleWrapper, With<PrimaryWindow>>,
    mut state: ResMut<EditorState>,
    mut dialogs: ResMut<DialogState>,
) {
    let parent_handle = primary_window_query.iter().next();

    if shortcut_just_pressed(&keys, state.keybinds.binding(ShortcutAction::OpenWorkspace)) {
        info!(
            "[dialog] Open-workspace shortcut detected (parent_handle: {}, has_pending: {})",
            parent_handle.is_some(),
            dialogs.pending.is_some()
        );
        open_workspace_dialog(&mut state, &mut dialogs, parent_handle);
    }

    if shortcut_just_pressed(&keys, state.keybinds.binding(ShortcutAction::SaveAs)) {
        info!(
            "[dialog] Save shortcut detected (parent_handle: {}, has_pending: {})",
            parent_handle.is_some(),
            dialogs.pending.is_some()
        );
        open_save_dialog(&mut state, &mut dialogs, parent_handle);
    }
}

fn open_workspace_dialog(
    state: &mut EditorState,
    dialogs: &mut DialogState,
    parent_handle: Option<&RawHandleWrapper>,
) {
    if dialogs.pending.is_some() {
        let pending_kind = dialogs
            .pending
            .as_ref()
            .map_or("unknown", PendingDialog::kind_name);
        warn!(
            "[dialog] Ignoring workspace request because {} dialog is already pending",
            pending_kind
        );
        state.status_message = "A file dialog is already open.".to_string();
        return;
    }

    info!(
        "[dialog] Starting workspace dialog request on thread {:?}",
        std::thread::current().id()
    );

    let mut dialog = AsyncFileDialog::new().set_title("Open Workspace Folder");

    if let Some(directory) = preferred_dialog_directory(state) {
        info!(
            "[dialog] Workspace dialog preferred directory: {}",
            directory.display()
        );
        dialog = dialog.set_directory(directory);
    } else {
        warn!("[dialog] No preferred directory found for workspace dialog");
    }

    dialog = attach_dialog_parent(dialog, parent_handle);

    info!("[dialog] Creating native workspace dialog future");
    let request = dialog.pick_folder();
    info!("[dialog] Native workspace future created; spawning task");

    let task = AsyncComputeTaskPool::get().spawn(async move {
        info!("[dialog] Workspace task awaiting picker result...");
        let result = request
            .await
            .map(|file_handle| file_handle.path().to_path_buf());
        match &result {
            Some(path) => info!("[dialog] Workspace task received path: {}", path.display()),
            None => info!("[dialog] Workspace task returned: canceled"),
        }
        result
    });

    dialogs.begin_pending(PendingDialog::Workspace(task));
    info!("[dialog] Workspace dialog task spawned");
    state.status_message = "Opening workspace picker...".to_string();
}

fn open_save_dialog(
    state: &mut EditorState,
    dialogs: &mut DialogState,
    parent_handle: Option<&RawHandleWrapper>,
) {
    if dialogs.pending.is_some() {
        let pending_kind = dialogs
            .pending
            .as_ref()
            .map_or("unknown", PendingDialog::kind_name);
        warn!(
            "[dialog] Ignoring save request because {} dialog is already pending",
            pending_kind
        );
        state.status_message = "A file dialog is already open.".to_string();
        return;
    }

    info!(
        "[dialog] Starting save dialog request on thread {:?}",
        std::thread::current().id()
    );

    let mut dialog = AsyncFileDialog::new()
        .set_title("Save Script File")
        .add_filter("Script files", &["fountain", "txt", "md"]);

    if let Some(directory) = preferred_dialog_directory(state) {
        info!(
            "[dialog] Save dialog preferred directory: {}",
            directory.display()
        );
        dialog = dialog.set_directory(directory);
    } else {
        warn!("[dialog] No preferred directory found for save dialog");
    }

    let default_name = state
        .paths
        .save_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("script.fountain")
        .to_string();

    info!("[dialog] Save dialog default filename: {}", default_name);
    dialog = dialog.set_file_name(default_name.as_str());
    dialog = attach_dialog_parent(dialog, parent_handle);

    info!("[dialog] Creating native save dialog future");
    let request = dialog.save_file();
    info!("[dialog] Native save future created; spawning task");

    let task = AsyncComputeTaskPool::get().spawn(async move {
        info!("[dialog] Save task awaiting picker result...");
        let result = request
            .await
            .map(|file_handle| file_handle.path().to_path_buf());
        match &result {
            Some(path) => info!("[dialog] Save task received path: {}", path.display()),
            None => info!("[dialog] Save task returned: canceled"),
        }
        result
    });

    dialogs.begin_pending(PendingDialog::Save(task));
    info!("[dialog] Save dialog task spawned");
    state.status_message = "Opening save dialog...".to_string();
}

fn attach_dialog_parent(
    dialog: AsyncFileDialog,
    parent_handle: Option<&RawHandleWrapper>,
) -> AsyncFileDialog {
    let Some(parent_handle) = parent_handle else {
        warn!("[dialog] No primary window handle found; opening unparented dialog");
        return dialog;
    };

    // SAFETY: This is called from Bevy update systems on the main app thread.
    let handle = unsafe { parent_handle.get_handle() };
    info!("[dialog] Attached dialog parent to primary window handle");
    dialog.set_parent(&handle)
}

fn resolve_dialog_results(mut state: ResMut<EditorState>, mut dialogs: ResMut<DialogState>) {
    let Some(pending) = dialogs.pending.as_mut() else {
        return;
    };
    let pending_kind = pending.kind_name();

    enum DialogResult {
        Workspace(Option<PathBuf>),
        Save(Option<PathBuf>),
    }

    let finished = match pending {
        PendingDialog::Workspace(task) => {
            future::block_on(future::poll_once(task)).map(DialogResult::Workspace)
        }
        PendingDialog::Save(task) => {
            future::block_on(future::poll_once(task)).map(DialogResult::Save)
        }
    };

    dialogs.poll_count = dialogs.poll_count.saturating_add(1);

    let now = Instant::now();
    let should_log_watchdog = dialogs.last_watchdog_log_at.map_or(true, |last| {
        now.duration_since(last) >= Duration::from_secs(2)
    });
    if should_log_watchdog {
        if let Some(opened_at) = dialogs.opened_at {
            let elapsed_ms = opened_at.elapsed().as_millis();
            info!(
                "[dialog] {} dialog pending for {}ms (poll_count={})",
                pending_kind, elapsed_ms, dialogs.poll_count
            );
        }
        dialogs.last_watchdog_log_at = Some(now);
    }

    let Some(result) = finished else {
        return;
    };

    let elapsed_ms = dialogs
        .opened_at
        .map_or(0_u128, |opened_at| opened_at.elapsed().as_millis());
    info!(
        "[dialog] {} dialog future resolved after {}ms (poll_count={})",
        pending_kind, elapsed_ms, dialogs.poll_count
    );

    dialogs.clear_pending();

    match result {
        DialogResult::Workspace(Some(path)) => {
            info!("[dialog] Opening selected workspace path: {}", path.display());
            state.set_workspace_root(path);
        }
        DialogResult::Workspace(None) => {
            info!("[dialog] Workspace dialog canceled by user");
            state.status_message = "Workspace open canceled.".to_string();
        }
        DialogResult::Save(Some(path)) => {
            info!("[dialog] Saving to selected path: {}", path.display());
            state.save_to_path(path);
        }
        DialogResult::Save(None) => {
            info!("[dialog] Save dialog canceled by user");
            state.status_message = "Save canceled.".to_string();
        }
    }
}

fn preferred_dialog_directory(state: &EditorState) -> Option<PathBuf> {
    state
        .workspace_root
        .clone()
        .or_else(|| {
            state
                .paths
                .load_path
                .parent()
                .map(|path| path.to_path_buf())
        })
        .or_else(|| {
            state
                .paths
                .save_path
                .parent()
                .map(|path| path.to_path_buf())
        })
}
