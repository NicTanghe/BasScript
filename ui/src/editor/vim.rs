fn handle_vim_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    body_query: Query<(&PanelBody, &ComputedNode)>,
    text_layout_query: Query<(&CanvasRenderedNodeText, &TextLayoutInfo, &ComputedNode)>,
    mut repeat: ResMut<NavigationRepeatState>,
    mut state: ResMut<EditorState>,
) {
    if !state.vim_enabled {
        return;
    }

    if state.document_format == DocumentFormat::Canvas {
        handle_canvas_vim_input(
            &mut keyboard_inputs,
            &keys,
            &time,
            &text_layout_query,
            &mut repeat,
            &mut state,
        );
        return;
    }

    if state.workspace_prompt.is_some() || state.command_menu.is_some() || state.workspace_focused {
        reset_vim_repeat(&mut repeat);
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

    if keys.just_pressed(KeyCode::Escape) {
        vim_enter_normal_mode(&mut state, true);
        reset_vim_repeat(&mut repeat);
        return;
    }

    if state.vim_mode == VimMode::Insert {
        return;
    }

    if text_input_modifier_pressed(&keys) {
        reset_vim_repeat(&mut repeat);
        return;
    }

    for input in keyboard_inputs.read() {
        if input.state.is_pressed()
            && state.vim_mode == VimMode::Normal
            && !keys.pressed(KeyCode::Space)
            && keyboard_input_text_is(input, ":")
        {
            state.open_command_menu();
            reset_vim_repeat(&mut repeat);
            return;
        }
    }

    if handle_vim_non_movement_command(&keys, &mut state, processed_panel_size, visible_lines) {
        reset_vim_repeat(&mut repeat);
        return;
    }

    let moved = repeat_key_input(
        &keys,
        &time,
        &mut repeat,
        just_pressed_vim_movement_key,
        held_vim_movement_key,
        |keys, key| keys.pressed(key) && vim_movement_key_to_arrow(key).is_some(),
        |key| move_vim_cursor(&mut state, key, processed_panel_size),
    );

    if moved {
        state.vim_pending_operator = None;
        apply_cursor_follow_scroll_policy(&mut state, processed_panel_size, visible_lines);
    }
}

fn handle_canvas_vim_input(
    keyboard_inputs: &mut MessageReader<KeyboardInput>,
    keys: &ButtonInput<KeyCode>,
    time: &Time,
    text_layout_query: &Query<(&CanvasRenderedNodeText, &TextLayoutInfo, &ComputedNode)>,
    repeat: &mut NavigationRepeatState,
    state: &mut EditorState,
) {
    if state.workspace_prompt.is_some() || state.command_menu.is_some() || state.workspace_focused {
        reset_vim_repeat(repeat);
        return;
    }

    let has_active_text_node = state.canvas_editing_node_id.is_some();

    if keys.just_pressed(KeyCode::Escape) {
        if state.vim_mode == VimMode::Insert {
            canvas_vim_enter_normal_mode(state, true);
            state.canvas_version = state.canvas_version.saturating_add(1);
        } else if has_active_text_node {
            state.clear_canvas_text_edit();
            state.status_message = "Canvas text card deselected.".to_string();
        }
        reset_vim_repeat(repeat);
        return;
    }

    if state.vim_mode == VimMode::Insert {
        return;
    }

    if text_input_modifier_pressed(keys) {
        reset_vim_repeat(repeat);
        return;
    }

    for input in keyboard_inputs.read() {
        if input.state.is_pressed()
            && state.vim_mode == VimMode::Normal
            && !keys.pressed(KeyCode::Space)
            && keyboard_input_text_is(input, ":")
        {
            state.open_command_menu();
            reset_vim_repeat(repeat);
            return;
        }
    }

    if !has_active_text_node {
        reset_vim_repeat(repeat);
        return;
    }

    let Some(mut document) = state.active_canvas_text_document() else {
        state.clear_canvas_text_edit();
        reset_vim_repeat(repeat);
        return;
    };
    let Some(node_id) = state.canvas_editing_node_id.clone() else {
        reset_vim_repeat(repeat);
        return;
    };
    let active_layout = active_canvas_text_layout(state, text_layout_query);
    let zoom = state.zoom;

    if state.vim_mode == VimMode::Normal && keys.just_pressed(KeyCode::KeyI) {
        canvas_vim_enter_insert_mode(state);
        state.canvas_text_suppress_next_insert_input = true;
        state.canvas_version = state.canvas_version.saturating_add(1);
        reset_vim_repeat(repeat);
        return;
    }

    if state.vim_mode == VimMode::Normal && keys.just_pressed(KeyCode::KeyV) {
        if shift_modifier_pressed(keys) {
            canvas_vim_enter_visual_line_mode(state, &document);
        } else {
            canvas_vim_enter_visual_char_mode(state);
        }
        reset_vim_repeat(repeat);
        return;
    }

    if canvas_vim_handle_edit_command(keys, state, &node_id, &mut document) {
        reset_vim_repeat(repeat);
        return;
    }

    let extend_selection = matches!(state.vim_mode, VimMode::VisualChar | VimMode::VisualLine);
    let moved = repeat_key_input(
        keys,
        time,
        repeat,
        just_pressed_vim_movement_key,
        held_vim_movement_key,
        |keys, key| keys.pressed(key) && vim_movement_key_to_arrow(key).is_some(),
        |key| {
            canvas_vim_move_text_cursor(
                state,
                &document,
                key,
                extend_selection,
                active_layout,
                zoom,
            )
        },
    );

    if moved {
        state.vim_pending_operator = None;
    }
}

fn canvas_vim_enter_normal_mode(state: &mut EditorState, clear_selection: bool) {
    state.vim_mode = VimMode::Normal;
    state.vim_pending_operator = None;
    state.vim_visual_anchor = None;
    state.vim_visual_head = None;
    if clear_selection {
        state.canvas_text_selection_anchor = None;
    }
    state.pending_space_insert = false;
    state.pending_space_combo_canceled = false;
    state.status_message = "Vim normal mode.".to_string();
}

fn canvas_vim_enter_insert_mode(state: &mut EditorState) {
    state.vim_mode = VimMode::Insert;
    state.vim_pending_operator = None;
    state.vim_visual_anchor = None;
    state.vim_visual_head = None;
    state.canvas_text_selection_anchor = None;
    state.status_message = "Vim insert mode.".to_string();
}

fn canvas_vim_enter_visual_char_mode(state: &mut EditorState) {
    let cursor = state.canvas_text_cursor.position;
    state.vim_mode = VimMode::VisualChar;
    state.vim_pending_operator = None;
    state.vim_visual_anchor = None;
    state.vim_visual_head = None;
    state.canvas_text_selection_anchor = Some(cursor);
    state.canvas_version = state.canvas_version.saturating_add(1);
    state.status_message = "Vim visual mode.".to_string();
}

fn canvas_vim_enter_visual_line_mode(state: &mut EditorState, document: &Document) {
    let line = state
        .canvas_text_cursor
        .position
        .line
        .min(document.line_count().saturating_sub(1));
    let start = Position { line, column: 0 };
    let end = Position {
        line,
        column: document.line_len_chars(line),
    };
    state.vim_mode = VimMode::VisualLine;
    state.vim_pending_operator = None;
    state.vim_visual_anchor = Some(start);
    state.vim_visual_head = Some(end);
    state.canvas_text_selection_anchor = Some(start);
    state.canvas_text_cursor.set_position(end);
    state.canvas_version = state.canvas_version.saturating_add(1);
    state.status_message = "Vim visual line mode.".to_string();
}

fn canvas_vim_move_text_cursor(
    state: &mut EditorState,
    document: &Document,
    key: KeyCode,
    extend_selection: bool,
    layout: Option<(&TextLayoutInfo, f32)>,
    zoom: f32,
) -> bool {
    let Some(arrow) = vim_movement_key_to_arrow(key) else {
        return false;
    };

    if matches!(state.vim_mode, VimMode::VisualLine) {
        return canvas_vim_move_visual_line(state, document, arrow);
    }

    move_canvas_text_cursor_by_key(state, document, arrow, extend_selection, layout, zoom)
}

fn canvas_vim_move_visual_line(
    state: &mut EditorState,
    document: &Document,
    arrow: KeyCode,
) -> bool {
    let current = state
        .vim_visual_head
        .unwrap_or(state.canvas_text_cursor.position);
    let current_line = current.line.min(document.line_count().saturating_sub(1));
    let next_line = match arrow {
        KeyCode::ArrowUp => current_line.saturating_sub(1),
        KeyCode::ArrowDown => (current_line + 1).min(document.line_count().saturating_sub(1)),
        _ => current_line,
    };
    if next_line == current_line {
        return false;
    }
    let anchor = state.vim_visual_anchor.unwrap_or(Position {
        line: current_line,
        column: 0,
    });
    let start_line = anchor.line.min(next_line);
    let end_line = anchor.line.max(next_line);
    let start = Position {
        line: start_line,
        column: 0,
    };
    let end = Position {
        line: end_line,
        column: document.line_len_chars(end_line),
    };
    state.vim_visual_head = Some(Position {
        line: next_line,
        column: document.line_len_chars(next_line),
    });
    state.canvas_text_selection_anchor = Some(start);
    state.canvas_text_cursor.set_position(end);
    state.canvas_version = state.canvas_version.saturating_add(1);
    true
}

fn canvas_vim_handle_edit_command(
    keys: &ButtonInput<KeyCode>,
    state: &mut EditorState,
    node_id: &str,
    document: &mut Document,
) -> bool {
    match state.vim_mode {
        VimMode::Normal => canvas_vim_handle_normal_command(keys, state, node_id, document),
        VimMode::VisualChar | VimMode::VisualLine => {
            canvas_vim_handle_visual_command(keys, state, node_id, document)
        }
        VimMode::Insert => false,
    }
}

fn canvas_vim_handle_normal_command(
    keys: &ButtonInput<KeyCode>,
    state: &mut EditorState,
    node_id: &str,
    document: &mut Document,
) -> bool {
    if keys.just_pressed(KeyCode::KeyY) {
        if state.vim_pending_operator == Some(VimPendingOperator::Yank) {
            state.vim_pending_operator = None;
            let line = state
                .canvas_text_cursor
                .position
                .line
                .min(document.line_count().saturating_sub(1));
            state.vim_register = Some(VimRegister::Linewise(
                document.line(line).unwrap_or("").to_string(),
            ));
            state.status_message = "Yanked canvas line.".to_string();
        } else {
            state.vim_pending_operator = Some(VimPendingOperator::Yank);
            state.status_message = "Vim: y".to_string();
        }
        return true;
    }

    if keys.just_pressed(KeyCode::KeyD) {
        if state.vim_pending_operator == Some(VimPendingOperator::Delete) {
            state.vim_pending_operator = None;
            canvas_delete_current_text_line(state, document);
            canvas_commit_text_change(state, node_id, document, "Deleted canvas line.");
        } else {
            state.vim_pending_operator = Some(VimPendingOperator::Delete);
            state.status_message = "Vim: d".to_string();
        }
        return true;
    }

    if keys.just_pressed(KeyCode::KeyP) {
        if canvas_paste_vim_register(state, document) {
            canvas_commit_text_change(state, node_id, document, "Pasted in canvas text.");
        }
        return true;
    }

    false
}

fn canvas_vim_handle_visual_command(
    keys: &ButtonInput<KeyCode>,
    state: &mut EditorState,
    node_id: &str,
    document: &mut Document,
) -> bool {
    if keys.just_pressed(KeyCode::KeyY) {
        let Some((start, end)) = state.canvas_text_selection_bounds(document) else {
            state.status_message = "Nothing selected.".to_string();
            return true;
        };
        let text = document_text_range(document, start, end);
        state.vim_register = Some(if state.vim_mode == VimMode::VisualLine {
            VimRegister::Linewise(text)
        } else {
            VimRegister::Characterwise(text)
        });
        canvas_vim_enter_normal_mode(state, true);
        state.status_message = "Yanked canvas selection.".to_string();
        return true;
    }

    if keys.just_pressed(KeyCode::KeyD) {
        if state.delete_canvas_text_selection(document) {
            canvas_commit_text_change(state, node_id, document, "Deleted canvas selection.");
        } else {
            state.status_message = "Nothing selected.".to_string();
        }
        canvas_vim_enter_normal_mode(state, true);
        return true;
    }

    false
}

fn canvas_delete_current_text_line(state: &mut EditorState, document: &mut Document) {
    let line = state
        .canvas_text_cursor
        .position
        .line
        .min(document.line_count().saturating_sub(1));
    let line_count = document.line_count();
    let start = Position { line, column: 0 };
    let end = if line + 1 < line_count {
        Position {
            line: line + 1,
            column: 0,
        }
    } else {
        Position {
            line,
            column: document.line_len_chars(line),
        }
    };
    let deleted = document_text_range(document, start, end);
    state.vim_register = Some(VimRegister::Linewise(deleted));
    let next = document.delete_range(start, end);
    state.canvas_text_cursor.set_position(next);
    state.canvas_text_selection_anchor = None;
}

fn canvas_paste_vim_register(state: &mut EditorState, document: &mut Document) -> bool {
    let Some(register) = state.vim_register.clone() else {
        state.status_message = "Vim register is empty.".to_string();
        return false;
    };

    state.delete_canvas_text_selection(document);
    let insert_position = state.canvas_text_cursor.position;
    let inserted = match register {
        VimRegister::Characterwise(text) => text,
        VimRegister::Linewise(text) => format!("\n{text}"),
    };
    let next = document.insert_text(insert_position, &inserted);
    state.canvas_text_cursor.set_position(next);
    state.canvas_text_selection_anchor = None;
    true
}

fn canvas_commit_text_change(
    state: &mut EditorState,
    node_id: &str,
    document: &Document,
    status_message: &str,
) {
    if let Some(snapshot) = state.canvas_text_edit_undo_snapshot.take() {
        state.push_undo_snapshot(snapshot);
    }
    if state.set_canvas_text_node_content(node_id, document.to_text()) {
        state.status_message = status_message.to_string();
    }
}

fn handle_vim_non_movement_command(
    keys: &ButtonInput<KeyCode>,
    state: &mut EditorState,
    processed_panel_size: Option<Vec2>,
    visible_lines: usize,
) -> bool {
    match state.vim_mode {
        VimMode::Normal => handle_vim_normal_command(keys, state, processed_panel_size, visible_lines),
        VimMode::VisualChar | VimMode::VisualLine => {
            handle_vim_visual_command(keys, state, processed_panel_size, visible_lines)
        }
        VimMode::Insert => false,
    }
}

fn handle_vim_normal_command(
    keys: &ButtonInput<KeyCode>,
    state: &mut EditorState,
    processed_panel_size: Option<Vec2>,
    visible_lines: usize,
) -> bool {
    if keys.just_pressed(KeyCode::KeyI) {
        vim_enter_insert_mode(state);
        return true;
    }

    if keys.just_pressed(KeyCode::KeyV) {
        if shift_modifier_pressed(keys) {
            vim_enter_visual_line_mode(state);
        } else {
            vim_enter_visual_char_mode(state);
        }
        return true;
    }

    if keys.just_pressed(KeyCode::KeyP) {
        state.vim_pending_operator = None;
        if let Some(dirty_line) = vim_paste_register(state) {
            state.reparse_with_dirty_hint(dirty_line);
            apply_cursor_follow_scroll_policy(state, processed_panel_size, visible_lines);
        }
        return true;
    }

    if keys.just_pressed(KeyCode::KeyY) {
        if state.vim_pending_operator == Some(VimPendingOperator::Yank) {
            state.vim_pending_operator = None;
            vim_yank_current_line(state);
        } else {
            state.vim_pending_operator = Some(VimPendingOperator::Yank);
            state.status_message = "Vim: y".to_string();
        }
        return true;
    }

    if keys.just_pressed(KeyCode::KeyD) {
        if state.vim_pending_operator == Some(VimPendingOperator::Delete) {
            state.vim_pending_operator = None;
            if let Some(dirty_line) = vim_delete_current_line(state) {
                state.reparse_with_dirty_hint(dirty_line);
                apply_cursor_follow_scroll_policy(state, processed_panel_size, visible_lines);
            }
        } else {
            state.vim_pending_operator = Some(VimPendingOperator::Delete);
            state.status_message = "Vim: d".to_string();
        }
        return true;
    }

    false
}

fn handle_vim_visual_command(
    keys: &ButtonInput<KeyCode>,
    state: &mut EditorState,
    processed_panel_size: Option<Vec2>,
    visible_lines: usize,
) -> bool {
    if keys.just_pressed(KeyCode::KeyY) {
        state.vim_pending_operator = None;
        vim_yank_visual_selection(state);
        vim_enter_normal_mode(state, true);
        return true;
    }

    if keys.just_pressed(KeyCode::KeyD) {
        state.vim_pending_operator = None;
        if let Some(dirty_line) = vim_delete_visual_selection(state) {
            state.reparse_with_dirty_hint(dirty_line);
            apply_cursor_follow_scroll_policy(state, processed_panel_size, visible_lines);
        }
        vim_enter_normal_mode(state, false);
        return true;
    }

    false
}

fn vim_enter_normal_mode(state: &mut EditorState, clear_selection: bool) {
    let visual_line_head = (state.vim_mode == VimMode::VisualLine)
        .then_some(state.vim_visual_head)
        .flatten();
    state.vim_mode = VimMode::Normal;
    state.vim_pending_operator = None;
    state.vim_visual_anchor = None;
    state.vim_visual_head = None;
    if clear_selection {
        if let Some(head) = visual_line_head {
            state.cursor.position = state.document.clamp_position(head);
            state.cursor.preferred_column = state.cursor.position.column;
        }
        state.selection_anchor = None;
    }
    state.pending_space_insert = false;
    state.pending_space_combo_canceled = false;
    state.status_message = "Vim normal mode.".to_string();
}

fn vim_enter_insert_mode(state: &mut EditorState) {
    state.vim_mode = VimMode::Insert;
    state.vim_pending_operator = None;
    state.vim_visual_anchor = None;
    state.vim_visual_head = None;
    state.selection_anchor = None;
    state.status_message = "Vim insert mode.".to_string();
}

fn vim_enter_visual_char_mode(state: &mut EditorState) {
    let cursor = state.cursor.position;
    state.vim_mode = VimMode::VisualChar;
    state.vim_pending_operator = None;
    state.vim_visual_anchor = Some(cursor);
    state.vim_visual_head = Some(cursor);
    state.selection_anchor = Some(cursor);
    state.status_message = "Vim visual mode.".to_string();
}

fn vim_enter_visual_line_mode(state: &mut EditorState) {
    let cursor = state.cursor.position;
    state.vim_mode = VimMode::VisualLine;
    state.vim_pending_operator = None;
    state.vim_visual_anchor = Some(cursor);
    state.vim_visual_head = Some(cursor);
    apply_vim_linewise_selection(state);
    state.status_message = "Vim visual line mode.".to_string();
}

fn move_vim_cursor(
    state: &mut EditorState,
    key: KeyCode,
    processed_panel_size: Option<Vec2>,
) -> bool {
    let Some(arrow) = vim_movement_key_to_arrow(key) else {
        return false;
    };

    match state.vim_mode {
        VimMode::Normal => move_cursor_by_arrow_key(state, arrow, false, processed_panel_size),
        VimMode::VisualChar => {
            let moved = move_cursor_by_arrow_key(state, arrow, true, processed_panel_size);
            state.vim_visual_head = Some(state.cursor.position);
            moved
        }
        VimMode::VisualLine => move_vim_visual_line_cursor(state, arrow, processed_panel_size),
        VimMode::Insert => false,
    }
}

fn move_vim_visual_line_cursor(
    state: &mut EditorState,
    arrow: KeyCode,
    processed_panel_size: Option<Vec2>,
) -> bool {
    let current_head = state.vim_visual_head.unwrap_or(state.cursor.position);
    state.cursor.position = current_head;
    state.cursor.preferred_column = current_head.column;

    let moved = move_cursor_by_arrow_key(state, arrow, false, processed_panel_size);
    let next_head = state.cursor.position;
    state.vim_visual_head = Some(next_head);
    apply_vim_linewise_selection(state);
    moved
}

fn apply_vim_linewise_selection(state: &mut EditorState) {
    let anchor_line = state
        .vim_visual_anchor
        .unwrap_or(state.cursor.position)
        .line
        .min(state.document.line_count().saturating_sub(1));
    let head_line = state
        .vim_visual_head
        .unwrap_or(state.cursor.position)
        .line
        .min(state.document.line_count().saturating_sub(1));
    let start_line = anchor_line.min(head_line);
    let end_line = anchor_line.max(head_line);
    let start = Position {
        line: start_line,
        column: 0,
    };
    let end = if end_line + 1 < state.document.line_count() {
        Position {
            line: end_line + 1,
            column: 0,
        }
    } else {
        Position {
            line: end_line,
            column: state.document.line_len_chars(end_line),
        }
    };

    state.selection_anchor = Some(start);
    state.cursor.position = end;
    state.cursor.preferred_column = end.column;
}

fn vim_yank_current_line(state: &mut EditorState) {
    let line = state
        .cursor
        .position
        .line
        .min(state.document.line_count().saturating_sub(1));
    let text = state.document.line(line).unwrap_or("").to_string();
    state.vim_register = Some(VimRegister::Linewise(text));
    state.status_message = "Yanked line.".to_string();
}

fn vim_yank_visual_selection(state: &mut EditorState) {
    match state.vim_mode {
        VimMode::VisualLine => {
            let text = vim_linewise_selection_text(state);
            state.vim_register = Some(VimRegister::Linewise(text));
            state.status_message = "Yanked lines.".to_string();
        }
        VimMode::VisualChar => {
            let Some((start, end)) = state.selection_bounds() else {
                state.status_message = "Nothing selected.".to_string();
                return;
            };
            let text = document_text_range(&state.document, start, end);
            state.vim_register = Some(VimRegister::Characterwise(text));
            state.status_message = "Yanked selection.".to_string();
        }
        VimMode::Normal | VimMode::Insert => {}
    }
}

fn vim_delete_current_line(state: &mut EditorState) -> Option<usize> {
    let snapshot = state.history_snapshot();
    let line = state
        .cursor
        .position
        .line
        .min(state.document.line_count().saturating_sub(1));
    let text = state.document.line(line).unwrap_or("").to_string();
    let line_count = state.document.line_count();
    let dirty_line = line.saturating_sub(1);

    if line_count <= 1 {
        let end = Position {
            line,
            column: state.document.line_len_chars(line),
        };
        state.document.delete_range(Position { line, column: 0 }, end);
        state.set_cursor(Position { line: 0, column: 0 }, true);
    } else if line + 1 < line_count {
        state.document.delete_range(
            Position { line, column: 0 },
            Position {
                line: line + 1,
                column: 0,
            },
        );
        state.set_cursor(
            Position {
                line: line.min(state.document.line_count().saturating_sub(1)),
                column: 0,
            },
            true,
        );
    } else {
        let previous_line = line.saturating_sub(1);
        let previous_len = state.document.line_len_chars(previous_line);
        let current_len = state.document.line_len_chars(line);
        state.document.delete_range(
            Position {
                line: previous_line,
                column: previous_len,
            },
            Position {
                line,
                column: current_len,
            },
        );
        state.set_cursor(
            Position {
                line: previous_line,
                column: 0,
            },
            true,
        );
    }

    state.vim_register = Some(VimRegister::Linewise(text));
    state.push_undo_snapshot(snapshot);
    state.status_message = "Deleted line.".to_string();
    Some(dirty_line)
}

fn vim_delete_visual_selection(state: &mut EditorState) -> Option<usize> {
    let (register, start, end) = match state.vim_mode {
        VimMode::VisualLine => {
            let text = vim_linewise_selection_text(state);
            let (start, end) = state.selection_bounds()?;
            (VimRegister::Linewise(text), start, end)
        }
        VimMode::VisualChar => {
            let (start, end) = state.selection_bounds()?;
            let text = document_text_range(&state.document, start, end);
            (VimRegister::Characterwise(text), start, end)
        }
        VimMode::Normal | VimMode::Insert => return None,
    };

    if start == end {
        state.status_message = "Nothing selected.".to_string();
        return None;
    }

    let snapshot = state.history_snapshot();
    let next = state.document.delete_range(start, end);
    state.vim_register = Some(register);
    state.set_cursor(next, true);
    state.push_undo_snapshot(snapshot);
    state.status_message = "Deleted selection.".to_string();
    Some(start.line)
}

fn vim_paste_register(state: &mut EditorState) -> Option<usize> {
    let Some(register) = state.vim_register.clone() else {
        state.status_message = "Vim register is empty.".to_string();
        return None;
    };

    let snapshot = state.history_snapshot();
    let current = state.cursor.position;
    let dirty_line = current.line;
    match register {
        VimRegister::Characterwise(text) => {
            let line_len = state.document.line_len_chars(current.line);
            let paste_position = if line_len == 0 {
                current
            } else {
                Position {
                    line: current.line,
                    column: current.column.saturating_add(1).min(line_len),
                }
            };
            let next = state.document.insert_text(paste_position, &text);
            state.set_cursor(next, true);
            state.status_message = "Pasted selection.".to_string();
        }
        VimRegister::Linewise(text) => {
            let line = current.line.min(state.document.line_count().saturating_sub(1));
            let column = state.document.line_len_chars(line);
            let inserted = format!("\n{text}");
            let next = state.document.insert_text(Position { line, column }, &inserted);
            state.set_cursor(next, true);
            state.status_message = "Pasted line.".to_string();
        }
    }

    state.push_undo_snapshot(snapshot);
    Some(dirty_line)
}

fn vim_linewise_selection_text(state: &EditorState) -> String {
    let anchor_line = state
        .vim_visual_anchor
        .unwrap_or(state.cursor.position)
        .line
        .min(state.document.line_count().saturating_sub(1));
    let head_line = state
        .vim_visual_head
        .unwrap_or(state.cursor.position)
        .line
        .min(state.document.line_count().saturating_sub(1));
    let start_line = anchor_line.min(head_line);
    let end_line = anchor_line.max(head_line);

    (start_line..=end_line)
        .map(|line| state.document.line(line).unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

fn document_text_range(document: &Document, start: Position, end: Position) -> String {
    let start = document.clamp_position(start);
    let end = document.clamp_position(end);
    if start.line > end.line || (start.line == end.line && start.column >= end.column) {
        return String::new();
    }

    if start.line == end.line {
        let line = document.line(start.line).unwrap_or("");
        let start_byte = vim_char_to_byte_index(line, start.column);
        let end_byte = vim_char_to_byte_index(line, end.column);
        return line[start_byte..end_byte].to_string();
    }

    let mut out = String::new();
    let first = document.line(start.line).unwrap_or("");
    let first_start = vim_char_to_byte_index(first, start.column);
    out.push_str(&first[first_start..]);

    for line_index in start.line.saturating_add(1)..end.line {
        out.push('\n');
        out.push_str(document.line(line_index).unwrap_or(""));
    }

    out.push('\n');
    let last = document.line(end.line).unwrap_or("");
    let last_end = vim_char_to_byte_index(last, end.column);
    out.push_str(&last[..last_end]);
    out
}

fn vim_char_to_byte_index(input: &str, column: usize) -> usize {
    if column == 0 {
        return 0;
    }

    input
        .char_indices()
        .map(|(byte, _)| byte)
        .nth(column)
        .unwrap_or(input.len())
}

fn just_pressed_vim_movement_key(keys: &ButtonInput<KeyCode>) -> Option<KeyCode> {
    vim_movement_keys()
        .into_iter()
        .find(|key| keys.just_pressed(*key))
}

fn held_vim_movement_key(keys: &ButtonInput<KeyCode>) -> Option<KeyCode> {
    vim_movement_keys()
        .into_iter()
        .find(|key| keys.pressed(*key))
}

fn vim_movement_keys() -> [KeyCode; 8] {
    [
        KeyCode::KeyJ,
        KeyCode::KeyK,
        KeyCode::KeyL,
        KeyCode::KeyH,
        KeyCode::ArrowDown,
        KeyCode::ArrowUp,
        KeyCode::ArrowRight,
        KeyCode::ArrowLeft,
    ]
}

fn vim_movement_key_to_arrow(key: KeyCode) -> Option<KeyCode> {
    match key {
        KeyCode::KeyJ | KeyCode::ArrowDown => Some(KeyCode::ArrowDown),
        KeyCode::KeyK | KeyCode::ArrowUp => Some(KeyCode::ArrowUp),
        KeyCode::KeyL | KeyCode::ArrowRight => Some(KeyCode::ArrowRight),
        KeyCode::KeyH | KeyCode::ArrowLeft => Some(KeyCode::ArrowLeft),
        _ => None,
    }
}

fn reset_vim_repeat(repeat: &mut NavigationRepeatState) {
    repeat.active_arrow = None;
    repeat.repeat_cooldown_secs = 0.0;
}
