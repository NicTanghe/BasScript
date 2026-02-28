fn handle_text_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    body_query: Query<(&PanelBody, &ComputedNode)>,
    mut state: ResMut<EditorState>,
) {
    if shortcut_modifier_pressed(&keys) {
        return;
    }

    let visible_lines = viewport_lines(
        &body_query,
        state.display_mode,
        state.measured_line_step,
        scaled_text_padding_y(&state),
    );
    let processed_panel_size = body_query
        .iter()
        .find(|(panel, _)| panel.kind == PanelKind::Processed)
        .map(|(_, computed)| computed.size() * computed.inverse_scale_factor());
    let mut edited = false;
    let mut dirty_from_line = None::<usize>;
    let mut undo_snapshot = None::<EditorHistorySnapshot>;

    for input in keyboard_inputs.read() {
        if !input.state.is_pressed() {
            continue;
        }

        let edit_intent = matches!(input.logical_key, Key::Enter | Key::Backspace | Key::Delete)
            || input
                .text
                .as_ref()
                .is_some_and(|text| !text.is_empty() && text.chars().all(is_printable_char));
        if !edit_intent {
            continue;
        }

        if undo_snapshot.is_none() {
            undo_snapshot = Some(state.history_snapshot());
        }

        let mut changed = false;
        let mut selection_deleted = false;

        if let Some(next) = state.delete_selection() {
            dirty_from_line = Some(dirty_from_line.map_or(next.line, |line| line.min(next.line)));
            changed = true;
            selection_deleted = true;
        }

        match &input.logical_key {
            Key::Enter => {
                let cursor_pos = state.cursor.position;
                let next = state.document.insert_newline(cursor_pos);
                state.set_cursor(next, true);
                dirty_from_line =
                    Some(dirty_from_line.map_or(cursor_pos.line, |line| line.min(cursor_pos.line)));
                changed = true;
            }
            Key::Backspace => {
                if selection_deleted {
                    if changed {
                        edited = true;
                    }
                    continue;
                }
                let cursor_pos = state.cursor.position;
                if cursor_pos.line > 0 || cursor_pos.column > 0 {
                    let next = state.document.backspace(cursor_pos);
                    state.set_cursor(next, true);
                    let dirty_candidate = cursor_pos.line.saturating_sub(1).min(next.line);
                    dirty_from_line = Some(
                        dirty_from_line.map_or(dirty_candidate, |line| line.min(dirty_candidate)),
                    );
                    changed = true;
                }
            }
            Key::Delete => {
                if selection_deleted {
                    if changed {
                        edited = true;
                    }
                    continue;
                }
                let cursor_pos = state.cursor.position;
                let line_len = state.document.line_len_chars(cursor_pos.line);
                let has_next_line = cursor_pos.line + 1 < state.document.line_count();
                if cursor_pos.column < line_len || has_next_line {
                    let next = state.document.delete(cursor_pos);
                    state.set_cursor(next, false);
                    dirty_from_line = Some(
                        dirty_from_line.map_or(cursor_pos.line, |line| line.min(cursor_pos.line)),
                    );
                    changed = true;
                }
            }
            _ => {
                if let Some(inserted_text) = &input.text {
                    if !inserted_text.is_empty() && inserted_text.chars().all(is_printable_char) {
                        let cursor_pos = state.cursor.position;
                        let next = state.document.insert_text(cursor_pos, inserted_text);
                        state.set_cursor(next, true);
                        dirty_from_line = Some(
                            dirty_from_line
                                .map_or(cursor_pos.line, |line| line.min(cursor_pos.line)),
                        );
                        changed = true;
                    }
                }
            }
        }

        if changed {
            edited = true;
        }
    }

    if edited {
        if let Some(snapshot) = undo_snapshot {
            state.push_undo_snapshot(snapshot);
        }
        state.reparse_with_dirty_hint(dirty_from_line.unwrap_or(0));
        apply_cursor_follow_scroll_policy(&mut state, processed_panel_size, visible_lines);
    }
}

fn handle_navigation_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    body_query: Query<(&PanelBody, &ComputedNode)>,
    mut navigation_repeat: ResMut<NavigationRepeatState>,
    mut state: ResMut<EditorState>,
) {
    let visible_lines = viewport_lines(
        &body_query,
        state.display_mode,
        state.measured_line_step,
        scaled_text_padding_y(&state),
    );
    let plain_panel_size = body_query
        .iter()
        .find(|(panel, _)| panel.kind == PanelKind::Plain)
        .map(|(_, computed)| computed.size() * computed.inverse_scale_factor());
    let processed_panel_size = body_query
        .iter()
        .find(|(panel, _)| panel.kind == PanelKind::Processed)
        .map(|(_, computed)| computed.size() * computed.inverse_scale_factor());
    state.clamp_horizontal_scrolls(plain_panel_size, processed_panel_size);
    let extend_selection = shift_modifier_pressed(&keys);
    let mut moved = false;

    if shortcut_modifier_pressed(&keys) {
        if shortcut_just_pressed(&keys, state.keybinds.binding(ShortcutAction::PlainView)) {
            state.set_display_mode(DisplayMode::Plain);
            state.status_message = format!("View mode: {}", state.display_mode.label());
            return;
        }

        if shortcut_just_pressed(&keys, state.keybinds.binding(ShortcutAction::ProcessedView)) {
            state.set_display_mode(DisplayMode::Processed);
            state.status_message = format!("View mode: {}", state.display_mode.label());
            return;
        }

        if shortcut_just_pressed(
            &keys,
            state
                .keybinds
                .binding(ShortcutAction::ProcessedRawCurrentLineView),
        ) {
            state.set_display_mode(DisplayMode::ProcessedRawCurrentLine);
            state.status_message = format!("View mode: {}", state.display_mode.label());
            return;
        }

        if shortcut_just_pressed(&keys, state.keybinds.binding(ShortcutAction::Redo)) {
            let changed = state.redo(visible_lines, plain_panel_size, processed_panel_size);

            if changed {
                state.status_message = "Redo".to_string();
                apply_cursor_follow_scroll_policy(&mut state, processed_panel_size, visible_lines);
            } else {
                state.status_message = "Nothing to redo.".to_string();
            }
            return;
        }

        if shortcut_just_pressed(&keys, state.keybinds.binding(ShortcutAction::Undo)) {
            let changed = state.undo(visible_lines, plain_panel_size, processed_panel_size);

            if changed {
                state.status_message = "Undo".to_string();
                apply_cursor_follow_scroll_policy(&mut state, processed_panel_size, visible_lines);
            } else {
                state.status_message = "Nothing to undo.".to_string();
            }
            return;
        }

        if shortcut_just_pressed(&keys, state.keybinds.binding(ShortcutAction::ZoomIn)) {
            let next_zoom = state.zoom + ZOOM_STEP;
            set_zoom_preserving_processed_anchor(&mut state, processed_panel_size, next_zoom);
            state.status_message = format!("Zoom: {}%", state.zoom_percent());
            let zoom_visible_lines = viewport_lines(
                &body_query,
                state.display_mode,
                state.measured_line_step,
                scaled_text_padding_y(&state),
            );
            state.clamp_scroll(zoom_visible_lines);
            state.clamp_horizontal_scrolls(plain_panel_size, processed_panel_size);
            return;
        }

        if shortcut_just_pressed(&keys, state.keybinds.binding(ShortcutAction::ZoomOut)) {
            let next_zoom = state.zoom - ZOOM_STEP;
            set_zoom_preserving_processed_anchor(&mut state, processed_panel_size, next_zoom);
            state.status_message = format!("Zoom: {}%", state.zoom_percent());
            let zoom_visible_lines = viewport_lines(
                &body_query,
                state.display_mode,
                state.measured_line_step,
                scaled_text_padding_y(&state),
            );
            state.clamp_scroll(zoom_visible_lines);
            state.clamp_horizontal_scrolls(plain_panel_size, processed_panel_size);
            return;
        }
    }

    let previous_active_arrow = navigation_repeat.active_arrow;
    if let Some(arrow) = just_pressed_navigation_arrow(&keys) {
        moved |= move_cursor_by_arrow_key(&mut state, arrow, extend_selection);
        navigation_repeat.active_arrow = Some(arrow);
        navigation_repeat.repeat_cooldown_secs = NAVIGATION_REPEAT_INITIAL_DELAY_SECS;
    } else {
        let active_arrow = navigation_repeat
            .active_arrow
            .filter(|arrow| keys.pressed(*arrow))
            .or_else(|| held_navigation_arrow(&keys));

        if active_arrow != previous_active_arrow {
            navigation_repeat.repeat_cooldown_secs = NAVIGATION_REPEAT_INITIAL_DELAY_SECS;
        }

        navigation_repeat.active_arrow = active_arrow;

        if let Some(arrow) = active_arrow {
            navigation_repeat.repeat_cooldown_secs -= time.delta_secs().max(0.0);
            while navigation_repeat.repeat_cooldown_secs <= 0.0 {
                moved |= move_cursor_by_arrow_key(&mut state, arrow, extend_selection);
                navigation_repeat.repeat_cooldown_secs += NAVIGATION_REPEAT_INTERVAL_SECS;
            }
        } else {
            navigation_repeat.repeat_cooldown_secs = 0.0;
        }
    }

    if keys.just_pressed(KeyCode::Home) {
        let line = state.cursor.position.line;
        state.set_cursor_with_selection(Position { line, column: 0 }, true, extend_selection);
        moved = true;
    }

    if keys.just_pressed(KeyCode::End) {
        let line = state.cursor.position.line;
        let column = state.document.line_len_chars(line);
        state.set_cursor_with_selection(Position { line, column }, true, extend_selection);
        moved = true;
    }

    let page_step = visible_lines.saturating_sub(1).max(1);

    if keys.just_pressed(KeyCode::PageUp) {
        let new_line = state.cursor.position.line.saturating_sub(page_step);
        let column = state
            .cursor
            .preferred_column
            .min(state.document.line_len_chars(new_line));

        state.set_cursor_with_selection(
            Position {
                line: new_line,
                column,
            },
            false,
            extend_selection,
        );
        moved = true;
    }

    if keys.just_pressed(KeyCode::PageDown) {
        let last_line = state.document.line_count().saturating_sub(1);
        let new_line = state
            .cursor
            .position
            .line
            .saturating_add(page_step)
            .min(last_line);
        let column = state
            .cursor
            .preferred_column
            .min(state.document.line_len_chars(new_line));

        state.set_cursor_with_selection(
            Position {
                line: new_line,
                column,
            },
            false,
            extend_selection,
        );
        moved = true;
    }

    if moved {
        apply_cursor_follow_scroll_policy(&mut state, processed_panel_size, visible_lines);
    }
}

fn just_pressed_navigation_arrow(keys: &ButtonInput<KeyCode>) -> Option<KeyCode> {
    [
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
    ]
    .into_iter()
    .find(|key| keys.just_pressed(*key))
}

fn held_navigation_arrow(keys: &ButtonInput<KeyCode>) -> Option<KeyCode> {
    [
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
    ]
    .into_iter()
    .find(|key| keys.pressed(*key))
}

fn move_cursor_by_arrow_key(
    state: &mut EditorState,
    arrow: KeyCode,
    extend_selection: bool,
) -> bool {
    let current = state.cursor.position;
    let next = match arrow {
        KeyCode::ArrowLeft => state.document.move_left(current),
        KeyCode::ArrowRight => state.document.move_right(current),
        KeyCode::ArrowUp => state.document.move_up(current, state.cursor.preferred_column),
        KeyCode::ArrowDown => state.document.move_down(current, state.cursor.preferred_column),
        _ => return false,
    };

    state.set_cursor_with_selection(
        next,
        !matches!(arrow, KeyCode::ArrowUp | KeyCode::ArrowDown),
        extend_selection,
    );
    next != current
}
