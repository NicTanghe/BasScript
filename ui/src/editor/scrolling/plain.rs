fn apply_plain_panel_horizontal_scroll(
    state: &mut EditorState,
    plain_panel_size: Option<Vec2>,
    horizontal_delta_px: f32,
) -> bool {
    if horizontal_delta_px.abs() <= f32::EPSILON {
        return false;
    }

    let max_scroll = plain_horizontal_scroll_max(state, plain_panel_size);
    let next_scroll = (state.plain_horizontal_scroll + horizontal_delta_px).clamp(0.0, max_scroll);
    let changed = (next_scroll - state.plain_horizontal_scroll).abs() > f32::EPSILON;
    state.plain_horizontal_scroll = next_scroll;
    changed
}

fn apply_plain_panel_vertical_scroll(
    state: &mut EditorState,
    line_delta: isize,
    visible_lines: usize,
) -> bool {
    if line_delta == 0 {
        return false;
    }

    let before = state.top_line;
    state.scroll_by(line_delta, visible_lines);
    state.top_line != before
}
