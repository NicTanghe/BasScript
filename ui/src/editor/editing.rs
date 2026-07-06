fn handle_text_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    capture: Res<LinkAutocompleteInputCapture>,
    body_query: Query<(&PanelBody, &ComputedNode)>,
    mut state: ResMut<EditorState>,
) {
    if capture.is_captured() {
        for _ in keyboard_inputs.read() {}
        state.pending_space_insert = false;
        state.pending_space_combo_canceled = false;
        return;
    }

    if state.workspace_focused
        || state.workspace_prompt.is_some()
        || state.command_menu.is_some()
        || state.markdown_metadata_input_active()
        || state.story_query_sheet.open
        || state.document_format == DocumentFormat::Canvas
        || (state.vim_enabled && state.vim_mode != VimMode::Insert)
    {
        state.close_link_autocomplete();
        state.pending_space_insert = false;
        state.pending_space_combo_canceled = false;
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

    if handle_document_clipboard_key_shortcut(
        &keys,
        &mut state,
        processed_panel_size,
        visible_lines,
    ) {
        for _ in keyboard_inputs.read() {}
        state.close_link_autocomplete();
        return;
    }

    let mut edited = false;
    let mut dirty_from_line = None::<usize>;
    let mut undo_snapshot = None::<EditorHistorySnapshot>;

    for input in keyboard_inputs.read() {
        if !input.state.is_pressed() {
            if input.key_code == KeyCode::Space && state.pending_space_insert {
                let should_insert_space = !state.pending_space_combo_canceled;
                state.pending_space_insert = false;
                state.pending_space_combo_canceled = false;

                if !should_insert_space {
                    continue;
                }

                if undo_snapshot.is_none() {
                    undo_snapshot = Some(state.history_snapshot());
                }

                let cursor_pos = state.cursor.position;
                let mut dirty_candidate = cursor_pos.line;
                if let Some(next) = state.delete_selection() {
                    dirty_candidate = next.line.min(dirty_candidate);
                }
                let insert_position = state.cursor.position;
                let next = state.document.insert_text(insert_position, " ");
                state.set_cursor(next, true);
                dirty_from_line =
                    Some(dirty_from_line.map_or(dirty_candidate, |line| line.min(dirty_candidate)));
                edited = true;
            }
            continue;
        }

        if handle_document_clipboard_input_shortcut(
            &keys,
            input,
            &mut state,
            processed_panel_size,
            visible_lines,
        ) {
            state.close_link_autocomplete();
            return;
        }

        if input.key_code == KeyCode::Space {
            if keys.just_pressed(KeyCode::Space)
                && !state.pending_space_insert
                && !text_input_should_skip_for_shortcut(&keys, input, &state.keybinds)
            {
                state.pending_space_insert = true;
            }
            continue;
        }

        if keys.pressed(KeyCode::Space) {
            state.pending_space_combo_canceled = true;
            continue;
        }

        if state.pending_space_insert {
            state.pending_space_combo_canceled = true;
            continue;
        }

        if text_input_should_skip_for_shortcut(&keys, input, &state.keybinds) {
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
        state.refresh_link_autocomplete_for_document_cursor();
        apply_cursor_follow_scroll_policy(&mut state, processed_panel_size, visible_lines);
    }
}

fn handle_navigation_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    capture: Res<LinkAutocompleteInputCapture>,
    body_query: Query<(&PanelBody, &ComputedNode)>,
    mut navigation_repeat: ResMut<NavigationRepeatState>,
    mut state: ResMut<EditorState>,
) {
    if capture.is_captured() {
        return;
    }

    if state.workspace_prompt.is_some()
        || state.command_menu.is_some()
        || state.markdown_metadata_input_active()
        || state.story_query_sheet.open
    {
        state.close_link_autocomplete();
        return;
    }

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

        return;
    }

    if state.vim_enabled && state.vim_mode != VimMode::Insert {
        state.close_link_autocomplete();
        return;
    }

    if state.workspace_focused {
        state.close_link_autocomplete();
        return;
    }

    if state.document_format == DocumentFormat::Canvas {
        return;
    }

    moved |= repeat_navigation_arrow_input(&keys, &time, &mut navigation_repeat, |arrow| {
        move_cursor_by_arrow_key(&mut state, arrow, extend_selection, processed_panel_size)
    });

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
        state.validate_link_autocomplete_context();
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

fn repeat_navigation_arrow_input(
    keys: &ButtonInput<KeyCode>,
    time: &Time,
    repeat: &mut NavigationRepeatState,
    on_key: impl FnMut(KeyCode) -> bool,
) -> bool {
    repeat_key_input(
        keys,
        time,
        repeat,
        just_pressed_navigation_arrow,
        held_navigation_arrow,
        |keys, key| keys.pressed(key),
        on_key,
    )
}

fn repeat_key_input(
    keys: &ButtonInput<KeyCode>,
    time: &Time,
    repeat: &mut NavigationRepeatState,
    just_pressed_key: impl Fn(&ButtonInput<KeyCode>) -> Option<KeyCode>,
    held_key: impl Fn(&ButtonInput<KeyCode>) -> Option<KeyCode>,
    key_still_pressed: impl Fn(&ButtonInput<KeyCode>, KeyCode) -> bool,
    mut on_key: impl FnMut(KeyCode) -> bool,
) -> bool {
    let previous_active_key = repeat.active_arrow;
    let mut moved = false;
    if let Some(key) = just_pressed_key(keys) {
        moved |= on_key(key);
        repeat.active_arrow = Some(key);
        repeat.repeat_cooldown_secs = NAVIGATION_REPEAT_INITIAL_DELAY_SECS;
    } else {
        let active_key = repeat
            .active_arrow
            .filter(|key| key_still_pressed(keys, *key))
            .or_else(|| held_key(keys));

        if active_key != previous_active_key {
            repeat.repeat_cooldown_secs = NAVIGATION_REPEAT_INITIAL_DELAY_SECS;
        }

        repeat.active_arrow = active_key;

        if let Some(key) = active_key {
            repeat.repeat_cooldown_secs -= time.delta_secs().max(0.0);
            while repeat.repeat_cooldown_secs <= 0.0 {
                moved |= on_key(key);
                repeat.repeat_cooldown_secs += NAVIGATION_REPEAT_INTERVAL_SECS;
            }
        } else {
            repeat.repeat_cooldown_secs = 0.0;
        }
    }

    moved
}

fn move_cursor_by_arrow_key(
    state: &mut EditorState,
    arrow: KeyCode,
    extend_selection: bool,
    processed_panel_size: Option<Vec2>,
) -> bool {
    if let Some(moved) =
        move_processed_cursor_by_visual_arrow_key(state, arrow, extend_selection, processed_panel_size)
    {
        return moved;
    }

    state.processed_preferred_column = None;
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

fn move_processed_cursor_by_visual_arrow_key(
    state: &mut EditorState,
    arrow: KeyCode,
    extend_selection: bool,
    processed_panel_size: Option<Vec2>,
) -> Option<bool> {
    let direction = match arrow {
        KeyCode::ArrowUp => -1,
        KeyCode::ArrowDown => 1,
        _ => return None,
    };
    if state.focused_panel != PanelKind::Processed || !state.panel_visible(PanelKind::Processed) {
        return None;
    }

    let panel_size = processed_panel_size?;
    let layout = processed_page_layout(panel_size, state);
    let processed_lines = processed_display_lines(
        state,
        layout.wrap_columns,
        layout.lines_per_page,
        layout.spacer_lines,
    );
    if processed_lines.is_empty() {
        state.processed_preferred_column = None;
        return Some(false);
    }

    let (current_visual_index, current_display_column) =
        processed_cursor_visual_index_and_display_column(state, &processed_lines)?;
    let preferred_display_column = state
        .processed_preferred_column
        .unwrap_or(current_display_column);
    state.processed_preferred_column = Some(preferred_display_column);

    let Some(target_visual_index) =
        adjacent_processed_visual_line_index(&processed_lines, current_visual_index, direction)
    else {
        return Some(false);
    };
    let Some(visual_line) = processed_lines.get(target_visual_index) else {
        return Some(false);
    };

    let display_column = preferred_display_column.min(visual_line.text.chars().count());
    let raw_column = processed_raw_column_from_display(visual_line, display_column);
    let next = Position {
        line: visual_line.source_line,
        column: raw_column.min(state.document.line_len_chars(visual_line.source_line)),
    };
    let current = state.cursor.position;

    state.set_cursor_with_selection(next, false, extend_selection);
    state.cursor.preferred_column = next.column;
    state.processed_preferred_column = Some(preferred_display_column);
    Some(next != current)
}

fn processed_cursor_visual_index_and_display_column(
    state: &EditorState,
    lines: &[ProcessedVisualLine],
) -> Option<(usize, usize)> {
    if let Some((visual_index, display_column, _)) =
        processed_cursor_visual_from_lines(state, lines)
    {
        return Some((visual_index, display_column));
    }

    let visual_index = first_visual_index_for_source_line(lines, state.cursor.position.line)?;
    let visual_line = lines.get(visual_index)?;
    Some((
        visual_index,
        processed_display_column_from_raw(visual_line, state.cursor.position.column),
    ))
}

fn adjacent_processed_visual_line_index(
    lines: &[ProcessedVisualLine],
    current_index: usize,
    direction: isize,
) -> Option<usize> {
    if direction < 0 {
        (0..current_index)
            .rev()
            .find(|index| lines.get(*index).is_some_and(|line| !line.is_spacer))
    } else {
        (current_index.saturating_add(1)..lines.len())
            .find(|index| lines.get(*index).is_some_and(|line| !line.is_spacer))
    }
}
