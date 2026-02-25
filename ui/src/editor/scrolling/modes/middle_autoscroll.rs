const MIDDLE_AUTOSCROLL_INDICATOR_SIZE_PX: f32 = 40.0;
const MIDDLE_AUTOSCROLL_DEAD_ZONE_PX: f32 = 12.0;
const MIDDLE_AUTOSCROLL_HORIZONTAL_GAIN_PX: f32 = 280.0;
const MIDDLE_AUTOSCROLL_VERTICAL_GAIN_LINES: f32 = 18.0;
const MIDDLE_AUTOSCROLL_ACCEL_EXPONENT: f32 = 1.25;
const MIDDLE_AUTOSCROLL_BASE_MAX_HORIZONTAL_SPEED_PX_PER_SEC: f32 = 2200.0;
const MIDDLE_AUTOSCROLL_BASE_MAX_VERTICAL_SPEED_LINES_PER_SEC: f32 = 72.0;
const MIDDLE_AUTOSCROLL_MAX_SPEED_MULTIPLIER: f32 = 3.0;

fn autoscroll_axis_speed(
    axis_offset: f32,
    dead_zone: f32,
    gain: f32,
    exponent: f32,
    max_speed: f32,
) -> f32 {
    let magnitude = (axis_offset.abs() - dead_zone).max(0.0);
    if magnitude <= f32::EPSILON {
        return 0.0;
    }

    let normalized = (magnitude / gain.max(1.0))
        .powf(exponent.max(1.0))
        .clamp(0.0, 1.0);
    axis_offset.signum() * normalized * max_speed.max(0.0)
}

fn handle_middle_mouse_autoscroll(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    panel_query: Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut middle_autoscroll: ResMut<MiddleAutoscrollState>,
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

    if mouse_buttons.just_pressed(MouseButton::Middle) {
        if middle_autoscroll.is_active() {
            return;
        }
        let (Some(panel), Some(cursor_position)) = (panel_context.hovered_panel, cursor_position)
        else {
            return;
        };
        middle_autoscroll.start(panel, cursor_position);
        state.focused_panel = panel;
        state.reset_blink();
        return;
    }

    if !middle_autoscroll.is_active() {
        return;
    }

    if mouse_buttons.just_released(MouseButton::Middle) {
        middle_autoscroll.stop();
        return;
    }

    if keys.just_pressed(KeyCode::Escape) || mouse_buttons.just_pressed(MouseButton::Right) {
        middle_autoscroll.stop();
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Left) {
        middle_autoscroll.stop();
        middle_autoscroll.suppress_next_left_click = true;
        return;
    }

    let Some(cursor_position) = cursor_position else {
        return;
    };
    let Some(active_panel) = middle_autoscroll.panel else {
        return;
    };
    let dt = time.delta_secs();
    if dt <= f32::EPSILON {
        return;
    }

    let delta = cursor_position - middle_autoscroll.anchor_cursor_position;
    let line_height = state.measured_line_step.max(1.0);
    let horizontal_max_speed =
        MIDDLE_AUTOSCROLL_BASE_MAX_HORIZONTAL_SPEED_PX_PER_SEC * MIDDLE_AUTOSCROLL_MAX_SPEED_MULTIPLIER;
    let vertical_max_speed =
        MIDDLE_AUTOSCROLL_BASE_MAX_VERTICAL_SPEED_LINES_PER_SEC * MIDDLE_AUTOSCROLL_MAX_SPEED_MULTIPLIER;
    let horizontal_px_per_sec = autoscroll_axis_speed(
        delta.x,
        MIDDLE_AUTOSCROLL_DEAD_ZONE_PX,
        MIDDLE_AUTOSCROLL_HORIZONTAL_GAIN_PX,
        MIDDLE_AUTOSCROLL_ACCEL_EXPONENT,
        horizontal_max_speed,
    );
    let vertical_dead_zone_lines = MIDDLE_AUTOSCROLL_DEAD_ZONE_PX / line_height;
    let vertical_lines_from_anchor = delta.y / line_height;
    let vertical_lines_per_sec = autoscroll_axis_speed(
        vertical_lines_from_anchor,
        vertical_dead_zone_lines,
        MIDDLE_AUTOSCROLL_VERTICAL_GAIN_LINES,
        MIDDLE_AUTOSCROLL_ACCEL_EXPONENT,
        vertical_max_speed,
    );
    let horizontal_delta_px = horizontal_px_per_sec * dt;
    let vertical_delta_lines = vertical_lines_per_sec * dt;
    if horizontal_delta_px.abs() <= f32::EPSILON && vertical_delta_lines.abs() <= f32::EPSILON {
        return;
    }

    let visible_lines = viewport_lines_from_panels(
        &panel_query,
        state.measured_line_step,
        scaled_text_padding_y(&state),
    );
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
            let total_lines = middle_autoscroll.plain_vertical_remainder_lines + vertical_delta_lines;
            let whole_lines = total_lines.trunc() as isize;
            middle_autoscroll.plain_vertical_remainder_lines = total_lines - whole_lines as f32;

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

fn sync_middle_autoscroll_indicator(
    middle_autoscroll: Res<MiddleAutoscrollState>,
    mut indicator_query: Query<&mut Node, With<MiddleAutoscrollIndicator>>,
) {
    let Ok(mut indicator_node) = indicator_query.single_mut() else {
        return;
    };

    if !middle_autoscroll.is_active() {
        indicator_node.display = Display::None;
        return;
    }

    let half = MIDDLE_AUTOSCROLL_INDICATOR_SIZE_PX * 0.5;
    indicator_node.display = Display::Flex;
    indicator_node.left = px(middle_autoscroll.anchor_cursor_position.x - half);
    indicator_node.top = px(middle_autoscroll.anchor_cursor_position.y - half);
}
