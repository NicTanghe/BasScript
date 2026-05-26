fn handle_mouse_scroll(
    mut mouse_wheels: MessageReader<MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    panel_query: Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
    mut state: ResMut<EditorState>,
) {
    let shift_horizontal = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let panel_context = gather_scroll_panels_context(&panel_query, &state);
    state.clamp_horizontal_scrolls(
        panel_context.plain_panel_size,
        panel_context.processed_panel_size,
    );

    if state.document_format == DocumentFormat::Canvas {
        handle_canvas_mouse_scroll(&mut mouse_wheels, &keys, &panel_context, &mut state);
        return;
    }

    if platform_shortcut_modifier_pressed(&keys) {
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
            set_zoom_preserving_processed_anchor(
                &mut state,
                panel_context.processed_panel_size,
                next_zoom,
            );
            state.status_message = format!("Zoom: {}%", state.zoom_percent());
            let visible_lines = viewport_lines_from_panels(
                &panel_query,
                state.display_mode,
                state.measured_line_step,
                scaled_text_padding_y(&state),
            );
            state.clamp_scroll(visible_lines);
            state.clamp_horizontal_scrolls(
                panel_context.plain_panel_size,
                panel_context.processed_panel_size,
            );
        }
        return;
    }

    let visible_lines = viewport_lines_from_panels(
        &panel_query,
        state.display_mode,
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

    let active_panel = panel_context
        .hovered_panel
        .unwrap_or_else(|| state.active_panel_for_display_mode());
    state.focused_panel = active_panel;
    let mut scrolled = false;

    if horizontal_delta_px.abs() > f32::EPSILON {
        scrolled |= match active_panel {
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
                    panel_context.processed_panel_size,
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

fn handle_canvas_mouse_scroll(
    mouse_wheels: &mut MessageReader<MouseWheel>,
    keys: &ButtonInput<KeyCode>,
    panel_context: &ScrollPanelsContext,
    state: &mut EditorState,
) {
    if panel_context.hovered_panel != Some(PanelKind::Processed) {
        for _ in mouse_wheels.read() {}
        return;
    }

    let shift_horizontal = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let mut dx = 0.0_f32;
    let mut dy = 0.0_f32;

    for wheel in mouse_wheels.read() {
        let scale = match wheel.unit {
            MouseScrollUnit::Line => CANVAS_SCROLL_STEP_PX,
            MouseScrollUnit::Pixel => 1.0,
        };
        let mut wheel_x = wheel.x * scale;
        let mut wheel_y = wheel.y * scale;
        if shift_horizontal && wheel_x.abs() <= f32::EPSILON {
            wheel_x = -wheel_y;
            wheel_y = 0.0;
        }
        dx += wheel_x;
        dy += wheel_y;
    }

    if platform_shortcut_modifier_pressed(keys) {
        if dy.abs() > f32::EPSILON {
            let old_zoom = state.zoom.max(CANVAS_ZOOM_MIN);
            let cursor_world = panel_context
                .processed_cursor_pos
                .map(|cursor| state.canvas_pan + cursor / old_zoom);
            state.set_zoom(state.zoom + (dy / CANVAS_SCROLL_STEP_PX) * ZOOM_STEP);
            if let Some((cursor, world)) = panel_context.processed_cursor_pos.zip(cursor_world) {
                state.canvas_pan = world - cursor / state.zoom.max(CANVAS_ZOOM_MIN);
            }
            state.status_message = format!("Canvas zoom: {}%", state.zoom_percent());
        }
        return;
    }

    let zoom = state.zoom.max(0.1);
    state.canvas_pan.x += -dx / zoom;
    state.canvas_pan.y += -dy / zoom;
}
