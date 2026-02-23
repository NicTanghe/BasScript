struct CtrlLeftDragScrollState {
    panel: PanelKind,
    last_cursor_position: Vec2,
    plain_vertical_remainder_lines: f32,
}

fn handle_ctrl_left_drag_scroll(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    middle_autoscroll: Res<MiddleAutoscrollState>,
    panel_query: Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut drag_state: Local<Option<CtrlLeftDragScrollState>>,
    mut state: ResMut<EditorState>,
) {
    let cursor_position = window_query
        .iter()
        .next()
        .and_then(Window::cursor_position);
    let panel_context = gather_scroll_panels_context(&panel_query);
    state.clamp_horizontal_scrolls(
        panel_context.plain_panel_size,
        panel_context.processed_panel_size,
    );
    if middle_autoscroll.is_active() {
        *drag_state = None;
        return;
    }

    let ctrl_pressed = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if !ctrl_pressed || !mouse_buttons.pressed(MouseButton::Left) {
        *drag_state = None;
        return;
    }

    let Some(cursor_position) = cursor_position else {
        return;
    };

    if mouse_buttons.just_pressed(MouseButton::Left) || drag_state.is_none() {
        let Some(panel) = panel_context.hovered_panel else {
            *drag_state = None;
            return;
        };
        state.focused_panel = panel;
        *drag_state = Some(CtrlLeftDragScrollState {
            panel,
            last_cursor_position: cursor_position,
            plain_vertical_remainder_lines: 0.0,
        });
        return;
    }

    let Some(drag_state) = drag_state.as_mut() else {
        return;
    };
    let delta = cursor_position - drag_state.last_cursor_position;
    drag_state.last_cursor_position = cursor_position;

    if delta.length_squared() <= f32::EPSILON {
        return;
    }

    let visible_lines = viewport_lines_from_panels(
        &panel_query,
        state.measured_line_step,
        scaled_text_padding_y(&state),
    );
    let line_height = state.measured_line_step.max(1.0);
    let horizontal_delta_px = delta.x;
    let vertical_delta_lines = delta.y / line_height;
    let mut scrolled = false;

    if horizontal_delta_px.abs() > f32::EPSILON {
        scrolled |= match drag_state.panel {
            PanelKind::Plain => apply_plain_panel_horizontal_scroll(
                &mut state,
                panel_context.plain_panel_size,
                horizontal_delta_px,
            ),
            PanelKind::Processed => apply_processed_panel_horizontal_scroll(
                &mut state,
                panel_context.processed_panel_size,
                horizontal_delta_px,
            ),
        };
    }

    match drag_state.panel {
        PanelKind::Plain => {
            let total_lines = drag_state.plain_vertical_remainder_lines + vertical_delta_lines;
            let whole_lines = total_lines.trunc() as isize;
            drag_state.plain_vertical_remainder_lines = total_lines - whole_lines as f32;

            if whole_lines != 0 {
                scrolled |= apply_plain_panel_vertical_scroll(&mut state, whole_lines, visible_lines);
                state.clamp_cursor_to_visible_range(visible_lines);
            }
        }
        PanelKind::Processed => {
            if vertical_delta_lines.abs() > f32::EPSILON {
                scrolled |= apply_processed_panel_vertical_scroll(
                    &mut state,
                    panel_context.processed_panel_size,
                    vertical_delta_lines,
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
