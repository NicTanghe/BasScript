const PROCESSED_HORIZONTAL_OVERSCROLL_FACTOR: f32 = 0.65;
const PROCESSED_HORIZONTAL_OVERSCROLL_MIN: f32 = 120.0;
const PROCESSED_SCROLL_BIAS_LIMIT_PX: f32 = 120_000.0;

fn processed_horizontal_scroll_bounds_with_overscroll(
    state: &EditorState,
    processed_panel_size: Option<Vec2>,
) -> (f32, f32) {
    let Some(panel_size) = processed_panel_size else {
        return (0.0, 0.0);
    };

    let geometry = processed_page_geometry(panel_size, state);
    let base_left = geometry.paper_left;
    let base_right = geometry.paper_left + geometry.paper_width;
    let overflow_left = (-base_left).max(0.0);
    let overflow_right = (base_right - panel_size.x).max(0.0);
    let overscroll = (panel_size.x * PROCESSED_HORIZONTAL_OVERSCROLL_FACTOR)
        .max(PROCESSED_HORIZONTAL_OVERSCROLL_MIN);
    (-(overflow_left + overscroll), overflow_right + overscroll)
}

fn apply_processed_panel_horizontal_scroll(
    state: &mut EditorState,
    processed_panel_size: Option<Vec2>,
    horizontal_delta_px: f32,
) -> bool {
    if horizontal_delta_px.abs() <= f32::EPSILON {
        return false;
    }

    let (min_scroll, max_scroll) =
        processed_horizontal_scroll_bounds_with_overscroll(state, processed_panel_size);
    let next_scroll =
        (state.processed_horizontal_scroll + horizontal_delta_px).clamp(min_scroll, max_scroll);
    let changed = (next_scroll - state.processed_horizontal_scroll).abs() > f32::EPSILON;
    state.processed_horizontal_scroll = next_scroll;
    changed
}

fn apply_processed_panel_vertical_scroll(
    state: &mut EditorState,
    processed_panel_size: Option<Vec2>,
    delta_lines: f32,
    visible_lines: usize,
) -> bool {
    if delta_lines.abs() <= f32::EPSILON {
        return false;
    }

    let line_height = scaled_line_height(state).max(1.0);
    let total_delta_px = delta_lines * line_height;
    let requested_whole_lines = delta_lines.trunc() as isize;

    let top_line_before = state.top_line as isize;
    let page_top_before = processed_anchor_page_top_for_state(state, processed_panel_size);

    if requested_whole_lines != 0 {
        state.scroll_by(requested_whole_lines, visible_lines);
    }

    let actual_whole_lines = state.top_line as isize - top_line_before;
    if let (Some(before), Some(after)) = (
        page_top_before,
        processed_anchor_page_top_for_state(state, processed_panel_size),
    ) {
        let desired_page_shift_px = actual_whole_lines as f32 * line_height;
        state.processed_zoom_anchor_bias_px += (before - after) - desired_page_shift_px;
    }

    let leftover_px = total_delta_px - actual_whole_lines as f32 * line_height;
    state.processed_zoom_anchor_bias_px -= leftover_px;
    state.processed_zoom_anchor_bias_px = state
        .processed_zoom_anchor_bias_px
        .clamp(-PROCESSED_SCROLL_BIAS_LIMIT_PX, PROCESSED_SCROLL_BIAS_LIMIT_PX);

    actual_whole_lines != 0 || leftover_px.abs() > f32::EPSILON
}
