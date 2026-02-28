const PROCESSED_HORIZONTAL_OVERSCROLL_FACTOR: f32 = 0.65;
const PROCESSED_HORIZONTAL_OVERSCROLL_MIN: f32 = 120.0;

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

    let Some(panel_size) = processed_panel_size else {
        return false;
    };

    let processed_layout = processed_page_layout(panel_size, state);
    let all_lines = processed_display_lines(
        state,
        processed_layout.wrap_columns,
        processed_layout.lines_per_page,
        processed_layout.spacer_lines,
    );
    if all_lines.is_empty() {
        state.processed_top_visual = 0;
        state.processed_top_line = 0;
        state.top_line = 0;
        state.processed_zoom_anchor_bias_px = 0.0;
        return false;
    }

    let line_height = scaled_line_height(state).max(1.0);
    let requested_whole_lines = delta_lines.trunc() as isize;
    let max_visual = all_lines.len().saturating_sub(1) as isize;
    let current_visual = state.processed_top_visual.min(max_visual as usize) as isize;
    let next_visual = (current_visual + requested_whole_lines).clamp(0, max_visual);
    let actual_whole_lines = next_visual - current_visual;
    state.processed_top_visual = next_visual as usize;

    let leftover_px = (delta_lines - actual_whole_lines as f32) * line_height;
    state.processed_zoom_anchor_bias_px -= leftover_px;

    while state.processed_zoom_anchor_bias_px <= -line_height
        && state.processed_top_visual < all_lines.len().saturating_sub(1)
    {
        state.processed_zoom_anchor_bias_px += line_height;
        state.processed_top_visual = state.processed_top_visual.saturating_add(1);
    }
    while state.processed_zoom_anchor_bias_px >= line_height && state.processed_top_visual > 0 {
        state.processed_zoom_anchor_bias_px -= line_height;
        state.processed_top_visual = state.processed_top_visual.saturating_sub(1);
    }

    state.processed_zoom_anchor_bias_px = state
        .processed_zoom_anchor_bias_px
        .clamp(-line_height, line_height);

    let source_line = all_lines
        .get(state.processed_top_visual)
        .map_or(0, |line| line.source_line)
        .min(state.document.line_count().saturating_sub(1));
    state.processed_top_line = source_line;
    state.clamp_processed_top_line();
    state.top_line = source_line.min(state.max_top_line(visible_lines));

    actual_whole_lines != 0 || leftover_px.abs() > f32::EPSILON
}

fn apply_cursor_follow_scroll_policy(
    state: &mut EditorState,
    processed_panel_size: Option<Vec2>,
    visible_lines: usize,
) {
    match state.focused_panel {
        PanelKind::Plain => {
            // Plain is the anchor: keep panels aligned deterministically with plain top-line.
            state.ensure_cursor_visible(visible_lines);
            state.processed_top_line = state.top_line;
            state.clamp_processed_top_line();
            state.processed_zoom_anchor_bias_px = 0.0;

            if let Some(panel_size) = processed_panel_size {
                let processed_layout = processed_page_layout(panel_size, state);
                let all_lines = processed_display_lines(
                    state,
                    processed_layout.wrap_columns,
                    processed_layout.lines_per_page,
                    processed_layout.spacer_lines,
                );
                if all_lines.is_empty() {
                    state.processed_top_visual = 0;
                } else {
                    state.processed_top_visual =
                        first_visual_index_for_source_line(&all_lines, state.processed_top_line)
                            .unwrap_or(0);
                }
            }
        }
        PanelKind::Processed => {
            // Processed is the anchor: adjust only plain top-line.
            state.ensure_cursor_visible(visible_lines);
            state.clamp_processed_top_line();
        }
    }
}
