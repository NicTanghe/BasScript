fn handle_mouse_scroll(
    mut mouse_wheels: MessageReader<MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    panel_query: Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
    mut state: ResMut<EditorState>,
) {
    let shift_horizontal = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let panel_context = gather_scroll_panels_context(&panel_query);
    state.clamp_horizontal_scrolls(
        panel_context.plain_panel_size,
        panel_context.processed_panel_size,
    );

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
            set_zoom_preserving_processed_anchor(
                &mut state,
                panel_context.processed_panel_size,
                next_zoom,
            );
            state.status_message = format!("Zoom: {}%", state.zoom_percent());
            let visible_lines = viewport_lines_from_panels(
                &panel_query,
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

    let active_panel = panel_context.hovered_panel.unwrap_or(PanelKind::Plain);
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
