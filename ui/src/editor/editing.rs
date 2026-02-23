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
    body_query: Query<(&PanelBody, &ComputedNode)>,
    mut state: ResMut<EditorState>,
) {
    let visible_lines = viewport_lines(
        &body_query,
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
    let mut moved = false;

    if shortcut_modifier_pressed(&keys) {
        if keys.just_pressed(KeyCode::KeyZ) {
            let redo = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
            let changed = if redo {
                state.redo(visible_lines, plain_panel_size, processed_panel_size)
            } else {
                state.undo(visible_lines, plain_panel_size, processed_panel_size)
            };

            if changed {
                state.status_message = if redo {
                    "Redo".to_string()
                } else {
                    "Undo".to_string()
                };
                apply_cursor_follow_scroll_policy(&mut state, processed_panel_size, visible_lines);
            } else {
                state.status_message = if redo {
                    "Nothing to redo.".to_string()
                } else {
                    "Nothing to undo.".to_string()
                };
            }
            return;
        }

        if keys.just_pressed(KeyCode::Equal) {
            let next_zoom = state.zoom + ZOOM_STEP;
            set_zoom_preserving_processed_anchor(&mut state, processed_panel_size, next_zoom);
            state.status_message = format!("Zoom: {}%", state.zoom_percent());
            let zoom_visible_lines = viewport_lines(
                &body_query,
                state.measured_line_step,
                scaled_text_padding_y(&state),
            );
            state.clamp_scroll(zoom_visible_lines);
            state.clamp_horizontal_scrolls(plain_panel_size, processed_panel_size);
            return;
        }

        if keys.just_pressed(KeyCode::Minus) {
            let next_zoom = state.zoom - ZOOM_STEP;
            set_zoom_preserving_processed_anchor(&mut state, processed_panel_size, next_zoom);
            state.status_message = format!("Zoom: {}%", state.zoom_percent());
            let zoom_visible_lines = viewport_lines(
                &body_query,
                state.measured_line_step,
                scaled_text_padding_y(&state),
            );
            state.clamp_scroll(zoom_visible_lines);
            state.clamp_horizontal_scrolls(plain_panel_size, processed_panel_size);
            return;
        }
    }

    if keys.just_pressed(KeyCode::ArrowLeft) {
        let next = state.document.move_left(state.cursor.position);
        state.set_cursor(next, true);
        moved = true;
    }

    if keys.just_pressed(KeyCode::ArrowRight) {
        let next = state.document.move_right(state.cursor.position);
        state.set_cursor(next, true);
        moved = true;
    }

    if keys.just_pressed(KeyCode::ArrowUp) {
        let next = state
            .document
            .move_up(state.cursor.position, state.cursor.preferred_column);
        state.set_cursor(next, false);
        moved = true;
    }

    if keys.just_pressed(KeyCode::ArrowDown) {
        let next = state
            .document
            .move_down(state.cursor.position, state.cursor.preferred_column);
        state.set_cursor(next, false);
        moved = true;
    }

    if keys.just_pressed(KeyCode::Home) {
        let line = state.cursor.position.line;
        state.set_cursor(Position { line, column: 0 }, true);
        moved = true;
    }

    if keys.just_pressed(KeyCode::End) {
        let line = state.cursor.position.line;
        let column = state.document.line_len_chars(line);
        state.set_cursor(Position { line, column }, true);
        moved = true;
    }

    let page_step = visible_lines.saturating_sub(1).max(1);

    if keys.just_pressed(KeyCode::PageUp) {
        let new_line = state.cursor.position.line.saturating_sub(page_step);
        let column = state
            .cursor
            .preferred_column
            .min(state.document.line_len_chars(new_line));

        state.set_cursor(
            Position {
                line: new_line,
                column,
            },
            false,
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

        state.set_cursor(
            Position {
                line: new_line,
                column,
            },
            false,
        );
        moved = true;
    }

    if moved {
        apply_cursor_follow_scroll_policy(&mut state, processed_panel_size, visible_lines);
    }
}

fn handle_mouse_scroll(
    mut mouse_wheels: MessageReader<MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    panel_query: Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
    mut state: ResMut<EditorState>,
) {
    let shift_horizontal = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let mut plain_panel_size = None;
    let mut processed_panel_size = None;
    let mut hovered_panel = None;
    for (panel, relative_cursor, computed) in panel_query.iter() {
        let logical_size = computed.size() * computed.inverse_scale_factor();
        match panel.kind {
            PanelKind::Plain => plain_panel_size = Some(logical_size),
            PanelKind::Processed => processed_panel_size = Some(logical_size),
        }
        if relative_cursor.cursor_over() {
            hovered_panel = Some(panel.kind);
        }
    }
    state.clamp_horizontal_scrolls(plain_panel_size, processed_panel_size);

    if shortcut_modifier_pressed(&keys) {
        let mut zoom_steps = 0.0_f32;

        for wheel in mouse_wheels.read() {
            let y = match wheel.unit {
                MouseScrollUnit::Line => wheel.y,
                MouseScrollUnit::Pixel => wheel.y / 120.0,
            };
            zoom_steps += y;
        }

        if zoom_steps.abs() > f32::EPSILON {
            let next_zoom = state.zoom + zoom_steps * ZOOM_STEP;
            set_zoom_preserving_processed_anchor(&mut state, processed_panel_size, next_zoom);
            state.status_message = format!("Zoom: {}%", state.zoom_percent());
            let visible_lines = viewport_lines_from_panels(
                &panel_query,
                state.measured_line_step,
                scaled_text_padding_y(&state),
            );
            state.clamp_scroll(visible_lines);
            state.clamp_horizontal_scrolls(plain_panel_size, processed_panel_size);
        }
        return;
    }

    let visible_lines = viewport_lines_from_panels(
        &panel_query,
        state.measured_line_step,
        scaled_text_padding_y(&state),
    );
    let mut plain_delta_lines: isize = 0;
    let mut processed_delta_lines = 0.0_f32;
    let mut horizontal_delta_px = 0.0_f32;

    for wheel in mouse_wheels.read() {
        let mut dx = wheel.x;
        let mut dy = wheel.y;
        if shift_horizontal && dx.abs() <= f32::EPSILON {
            dx = -dy;
            dy = 0.0;
        }

        match wheel.unit {
            MouseScrollUnit::Line => {
                let vertical_lines = -dy;
                plain_delta_lines += vertical_lines.round() as isize;
                processed_delta_lines += vertical_lines;
                horizontal_delta_px += -dx * 32.0;
            }
            MouseScrollUnit::Pixel => {
                let vertical_lines = -dy / state.measured_line_step.max(1.0);
                plain_delta_lines += vertical_lines.round() as isize;
                processed_delta_lines += vertical_lines;
                horizontal_delta_px += -dx;
            }
        }
    }

    let active_panel = hovered_panel.unwrap_or(PanelKind::Plain);
    let mut scrolled = false;

    if horizontal_delta_px.abs() > f32::EPSILON {
        scrolled |= match active_panel {
            PanelKind::Plain => {
                apply_plain_panel_horizontal_scroll(&mut state, plain_panel_size, horizontal_delta_px)
            }
            PanelKind::Processed => apply_processed_panel_horizontal_scroll(
                &mut state,
                processed_panel_size,
                horizontal_delta_px,
            ),
        };
    }

    match active_panel {
        PanelKind::Plain => {
            if plain_delta_lines != 0 {
                scrolled |=
                    apply_plain_panel_vertical_scroll(&mut state, plain_delta_lines, visible_lines);
                state.clamp_cursor_to_visible_range(visible_lines);
            }
        }
        PanelKind::Processed => {
            if processed_delta_lines.abs() > f32::EPSILON {
                scrolled |= apply_processed_panel_vertical_scroll(
                    &mut state,
                    processed_panel_size,
                    processed_delta_lines,
                    visible_lines,
                );
                state.clamp_cursor_to_visible_range(visible_lines);
            }
        }
    }

    if scrolled {
        state.reset_blink();
    }
}

fn handle_mouse_click(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    panel_query: Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
    text_layout_query: Query<(&PanelText, &TextLayoutInfo)>,
    processed_text_layout_query: Query<
        (&ProcessedPaperText, &TextLayoutInfo, &ComputedNode),
        (Without<PanelText>, Without<PanelPaper>, Without<PanelCaret>, Without<PanelCanvas>),
    >,
    mut state: ResMut<EditorState>,
) {
    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let visible_lines = viewport_lines_from_panels(
        &panel_query,
        state.measured_line_step,
        scaled_text_padding_y(&state),
    );
    let plain_panel_size = panel_query
        .iter()
        .find(|(panel, _, _)| panel.kind == PanelKind::Plain)
        .map(|(_, _, computed)| computed.size() * computed.inverse_scale_factor());
    let processed_panel_size = panel_query
        .iter()
        .find(|(panel, _, _)| panel.kind == PanelKind::Processed)
        .map(|(_, _, computed)| computed.size() * computed.inverse_scale_factor());
    state.clamp_horizontal_scrolls(plain_panel_size, processed_panel_size);
    let processed_layout_info =
        processed_panel_size.map(|size| processed_page_layout(size, &state));
    let processed_wrap_columns = processed_layout_info.map_or(64, |layout| layout.wrap_columns);
    let processed_lines_per_page = processed_layout_info.map_or(40, |layout| layout.lines_per_page);
    let processed_spacer_lines = processed_layout_info.map_or(2, |layout| layout.spacer_lines);
    let processed_step_lines = processed_layout_info
        .map_or(processed_page_step_lines(), |layout| layout.page_step_lines)
        .max(1);
    let processed_view_capacity = processed_step_lines
        .saturating_mul(PROCESSED_PAPER_CAPACITY)
        .max(1);
    let plain_lines = visible_plain_lines(&state, visible_lines);
    let processed_all_lines = processed_cache_lines(
        &mut state,
        processed_wrap_columns,
        processed_lines_per_page,
        processed_spacer_lines,
    )
    .to_vec();
    if processed_all_lines.is_empty() {
        state.processed_top_visual = 0;
    } else {
        state.processed_top_visual = state
            .processed_top_visual
            .min(processed_all_lines.len().saturating_sub(1));
    }
    let processed_view = build_processed_view(
        &processed_all_lines,
        state.processed_top_visual,
        processed_step_lines,
        processed_view_capacity,
    );
    let first_visible_page = processed_view.start_index / processed_step_lines;
    let plain_layout = panel_layout_info(&text_layout_query, PanelKind::Plain);
    let plain_line_height = state.measured_line_step.max(1.0);
    let processed_line_height = scaled_line_height(&state).max(1.0);
    let plain_char_width = scaled_char_width(&state).max(1.0);
    let processed_char_width = scaled_char_width(&state).max(1.0);
    let plain_origin_x = scaled_text_padding_x(&state) - state.plain_horizontal_scroll;
    let plain_origin_y = scaled_text_padding_y(&state);
    let anchor_line_in_page = processed_anchor_line_in_page(&processed_view, processed_step_lines);
    let processed_anchor_offset_px =
        processed_anchor_scroll_offset_px(anchor_line_in_page, processed_line_height);
    let processed_zoom_bias_px = state.processed_zoom_anchor_bias_px;
    for (panel, relative_cursor, computed) in panel_query.iter() {
        if !relative_cursor.cursor_over() {
            continue;
        }

        let Some(normalized) = relative_cursor.normalized else {
            continue;
        };

        // Clicking anywhere in a panel selects its scroll-anchor policy.
        state.focused_panel = panel.kind;

        if state.document.is_empty() {
            state.set_cursor(Position::default(), true);
            break;
        }

        let inverse_scale = computed.inverse_scale_factor();
        let size = computed.size() * inverse_scale;
        let raw_x = (normalized.x + 0.5) * size.x;
        let raw_y = (normalized.y + 0.5) * size.y;
        let panel_x = raw_x;
        let panel_y = raw_y;
        if panel.kind == PanelKind::Processed {
            let Some(processed_layout) = processed_layout_info else {
                continue;
            };
            if processed_all_lines.is_empty() {
                continue;
            }

            let geometry = processed_layout.geometry;
            let processed_step_px = processed_page_step_px(&geometry, state.zoom);
            let text_left = geometry.text_left - state.processed_horizontal_scroll;
            let text_right = text_left + geometry.text_width;

            let mut clicked_page = None;
            for slot in 0..PROCESSED_PAPER_CAPACITY {
                let page_index = first_visible_page.saturating_add(slot);
                let page_top = processed_page_top_for_slot(
                    &geometry,
                    slot,
                    processed_step_px,
                    processed_anchor_offset_px,
                ) + processed_zoom_bias_px;
                let page_bottom = page_top + geometry.paper_height;

                if panel_y >= page_top && panel_y <= page_bottom {
                    clicked_page = Some((slot, page_index));
                    break;
                }
            }

            let Some((slot, page_index)) = clicked_page else {
                continue;
            };

            let text_top = processed_text_top_for_slot(
                &geometry,
                slot,
                processed_step_px,
                processed_anchor_offset_px,
            ) + processed_zoom_bias_px;
            let local_x = (panel_x - text_left).max(0.0);
            let local_y = (panel_y - text_top).max(0.0);
            let fallback_line_in_page = ((local_y / processed_line_height).floor().max(0.0)
                as usize)
                .min(processed_lines_per_page.saturating_sub(1));
            let fallback_column = if panel_x <= text_left {
                0
            } else {
                ((panel_x.min(text_right) - text_left) / processed_char_width)
                    .round()
                    .max(0.0) as usize
            };

            let (line_in_page, display_column) = processed_text_layout_query
                .iter()
                .find(|(paper_text, _, _)| paper_text.slot == slot)
                .map_or((fallback_line_in_page, fallback_column), |(_, layout, text_computed)| {
                    let inverse_scale = text_computed.inverse_scale_factor();
                    let line_in_page = line_index_from_layout_y(
                        layout,
                        local_y,
                        processed_lines_per_page.max(1),
                        inverse_scale,
                    )
                    .unwrap_or(fallback_line_in_page)
                    .min(processed_lines_per_page.saturating_sub(1));

                    let global_for_line = page_index
                        .saturating_mul(processed_step_lines)
                        .saturating_add(line_in_page)
                        .min(processed_all_lines.len().saturating_sub(1));
                    let display_line =
                        processed_all_lines.get(global_for_line).map_or("", |line| line.text.as_str());
                    let display_column = column_from_layout_x(
                        layout,
                        line_in_page,
                        local_x,
                        display_line,
                        inverse_scale,
                        processed_char_width,
                    )
                    .unwrap_or(fallback_column);
                    (line_in_page, display_column)
                });

            let global_index = page_index
                .saturating_mul(processed_step_lines)
                .saturating_add(line_in_page)
                .min(processed_all_lines.len().saturating_sub(1));
            let Some(global_index) =
                nearest_non_spacer_visual_index(&processed_all_lines, global_index)
            else {
                continue;
            };
            let Some(visual_line) = processed_all_lines.get(global_index) else {
                continue;
            };
            let raw_column = processed_raw_column_from_display(&state, visual_line, display_column);
            let line = visual_line.source_line;
            let max_col = state.document.line_len_chars(line);
            let column = raw_column.min(max_col);

            state.set_cursor(Position { line, column }, true);
            apply_cursor_follow_scroll_policy(&mut state, processed_panel_size, visible_lines);
            break;
        }

        let local_x = (panel_x - plain_origin_x).max(0.0);
        let local_y = (panel_y - plain_origin_y).max(0.0);
        let panel_line_count = plain_lines.len().max(1);
        let line_offset = plain_layout
            .and_then(|layout| {
                line_index_from_layout_y(layout, local_y, panel_line_count, inverse_scale)
            })
            .unwrap_or_else(|| {
                ((local_y / plain_line_height).floor().max(0.0) as usize)
                    .min(panel_line_count.saturating_sub(1))
            });
        let line = state
            .top_line
            .saturating_add(line_offset)
            .min(state.document.line_count().saturating_sub(1));
        let visible_offset = line.saturating_sub(state.top_line);
        let display_line = plain_lines
            .get(visible_offset)
            .map_or("", |line| line.as_str());
        let raw_column = plain_layout
            .and_then(|layout| {
                column_from_layout_x(
                    layout,
                    visible_offset,
                    local_x,
                    display_line,
                    inverse_scale,
                    plain_char_width,
                )
            })
            .unwrap_or_else(|| (local_x / plain_char_width).round().max(0.0) as usize);

        let max_col = state.document.line_len_chars(line);
        let column = raw_column.min(max_col);

        state.set_cursor(Position { line, column }, true);
        apply_cursor_follow_scroll_policy(&mut state, processed_panel_size, visible_lines);
        break;
    }
}
