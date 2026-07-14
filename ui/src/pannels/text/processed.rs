pub(crate) const PROCESSED_HORIZONTAL_OVERSCROLL_FACTOR: f32 = 0.65;
pub(crate) const PROCESSED_HORIZONTAL_OVERSCROLL_MIN: f32 = 120.0;

pub(crate) fn processed_horizontal_scroll_bounds_with_overscroll(
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

pub(crate) fn apply_processed_panel_horizontal_scroll(
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

pub(crate) fn apply_processed_panel_vertical_scroll(
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
        state.processed_header_scroll_progress = 0.0;
        state.processed_zoom_anchor_bias_px = 0.0;
        return false;
    }

    let line_height = scaled_line_height(state).max(1.0);
    let before = (
        state.processed_top_visual,
        state.processed_header_scroll_progress,
        state.processed_zoom_anchor_bias_px,
    );
    let delta_px = delta_lines * line_height;
    let _unconsumed_px = if delta_px > 0.0 {
        let remaining_px = apply_processed_header_scroll_px(state, delta_px);
        apply_processed_visual_scroll_px(state, &all_lines, remaining_px, line_height)
    } else {
        let remaining_px =
            apply_processed_visual_scroll_px(state, &all_lines, delta_px, line_height);
        apply_processed_header_scroll_px(state, remaining_px)
    };

    // The scroll is bounded at the beginning and end of the rendered document.
    // Any unconsumed input beyond those bounds is intentionally ignored.

    let source_line = all_lines
        .get(state.processed_top_visual)
        .map_or(0, |line| line.source_line)
        .min(state.document.line_count().saturating_sub(1));
    state.processed_top_line = source_line;
    state.clamp_processed_top_line();
    state.top_line = source_line.min(state.max_top_line(visible_lines));

    before
        != (
            state.processed_top_visual,
            state.processed_header_scroll_progress,
            state.processed_zoom_anchor_bias_px,
        )
}

pub(crate) fn apply_processed_header_scroll_px(state: &mut EditorState, delta_px: f32) -> f32 {
    if delta_px.abs() <= f32::EPSILON {
        return 0.0;
    }

    let header_height = markdown_metadata_full_header_offset(state);
    if header_height <= f32::EPSILON {
        state.processed_header_scroll_progress = 0.0;
        return delta_px;
    }

    let scrolled_px = header_height * state.processed_header_scroll_progress.clamp(0.0, 1.0);
    if delta_px > 0.0 {
        let applied_px = delta_px.min((header_height - scrolled_px).max(0.0));
        state.processed_header_scroll_progress =
            ((scrolled_px + applied_px) / header_height).clamp(0.0, 1.0);
        delta_px - applied_px
    } else {
        let applied_px = (-delta_px).min(scrolled_px.max(0.0));
        state.processed_header_scroll_progress =
            ((scrolled_px - applied_px) / header_height).clamp(0.0, 1.0);
        delta_px + applied_px
    }
}

pub(crate) fn apply_processed_visual_scroll_px(
    state: &mut EditorState,
    all_lines: &[ProcessedVisualLine],
    delta_px: f32,
    base_line_height: f32,
) -> f32 {
    if delta_px.abs() <= f32::EPSILON || all_lines.is_empty() {
        return delta_px;
    }

    let base_line_height = base_line_height.max(1.0);
    let max_visual = all_lines.len().saturating_sub(1);
    state.processed_top_visual = state.processed_top_visual.min(max_visual);
    state.processed_zoom_anchor_bias_px -= delta_px;

    if delta_px > 0.0 {
        while state.processed_top_visual < max_visual {
            let current_height =
                processed_visual_line_height_units(state, &all_lines[state.processed_top_visual])
                    .max(f32::EPSILON)
                    * base_line_height;
            if state.processed_zoom_anchor_bias_px > -current_height {
                break;
            }
            state.processed_zoom_anchor_bias_px += current_height;
            state.processed_top_visual = state.processed_top_visual.saturating_add(1);
        }

        let current_height =
            processed_visual_line_height_units(state, &all_lines[state.processed_top_visual])
                .max(f32::EPSILON)
                * base_line_height;
        if state.processed_top_visual == max_visual
            && state.processed_zoom_anchor_bias_px < -current_height
        {
            let overflow_px = -current_height - state.processed_zoom_anchor_bias_px;
            state.processed_zoom_anchor_bias_px = -current_height;
            return overflow_px;
        }
    } else {
        while state.processed_zoom_anchor_bias_px > 0.0 && state.processed_top_visual > 0 {
            state.processed_top_visual = state.processed_top_visual.saturating_sub(1);
            let previous_height =
                processed_visual_line_height_units(state, &all_lines[state.processed_top_visual])
                    .max(f32::EPSILON)
                    * base_line_height;
            state.processed_zoom_anchor_bias_px -= previous_height;
        }

        if state.processed_top_visual == 0 && state.processed_zoom_anchor_bias_px > 0.0 {
            let overflow_px = state.processed_zoom_anchor_bias_px;
            state.processed_zoom_anchor_bias_px = 0.0;
            return -overflow_px;
        }
    }

    0.0
}

pub(crate) fn apply_cursor_follow_scroll_policy(
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
                    state.processed_header_scroll_progress = 0.0;
                } else {
                    state.processed_top_visual =
                        first_visual_index_for_source_line(&all_lines, state.processed_top_line)
                            .unwrap_or(0);
                }
            }
            state.processed_header_scroll_progress = if state.top_line == 0 { 0.0 } else { 1.0 };
        }
        PanelKind::Processed => {
            // Processed is the anchor: keep the formatted caret visible and let plain follow.
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
                    state.processed_top_line = 0;
                    state.processed_header_scroll_progress = 0.0;
                } else if let Some((target_visual, _, _)) =
                    processed_cursor_visual_from_lines(state, &all_lines)
                {
                    let visible_visual_lines = visible_lines.max(1);
                    let max_visual = all_lines.len().saturating_sub(1);
                    let current_top = state.processed_top_visual.min(max_visual);
                    let past_bottom = current_top.saturating_add(visible_visual_lines);

                    if target_visual < current_top {
                        state.processed_top_visual = target_visual;
                        state.processed_zoom_anchor_bias_px = 0.0;
                    } else if target_visual >= past_bottom {
                        state.processed_top_visual =
                            target_visual.saturating_sub(visible_visual_lines.saturating_sub(1));
                        state.processed_zoom_anchor_bias_px = 0.0;
                    } else {
                        state.processed_top_visual = current_top;
                    }

                    let source_line = all_lines
                        .get(state.processed_top_visual)
                        .map_or(0, |line| line.source_line)
                        .min(state.document.line_count().saturating_sub(1));
                    state.processed_top_line = source_line;
                }
            }

            if state.processed_top_visual > 0 {
                state.processed_header_scroll_progress = 1.0;
            }

            state.ensure_cursor_visible(visible_lines);
            state.clamp_processed_top_line();
        }
    }
}
#[allow(unused_imports)]
use super::*;

#[cfg(test)]
mod processed_scroll_tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.01,
            "expected {expected}, got {actual}"
        );
    }

    fn markdown_state_with_front_matter() -> EditorState {
        let mut world = World::new();
        let mut state = EditorState::from_world(&mut world);
        state.document = Document::from_text(
            "---\nid: entity_eoghan_001\ntarget: eoghan\ntype: character\n---\n# Eoghan\nlead\n## looks\nA description.",
        );
        state.document_format = DocumentFormat::Markdown;
        state.display_mode = DisplayMode::Processed;
        state.processed_paginated = true;
        state.set_zoom(1.65);
        state.reparse();
        state
    }

    #[test]
    fn front_matter_header_is_consumed_without_a_page_jump() {
        let mut state = markdown_state_with_front_matter();
        let panel_size = Vec2::new(1_200.0, 900.0);
        let line_height = scaled_line_height(&state);
        let header_height = markdown_metadata_full_header_offset(&state);
        let first_delta_lines = header_height / line_height - 0.25;

        let before = processed_anchor_page_top_for_state(&mut state, Some(panel_size)).unwrap();
        assert!(apply_processed_panel_vertical_scroll(
            &mut state,
            Some(panel_size),
            first_delta_lines,
            40,
        ));
        let before_boundary =
            processed_anchor_page_top_for_state(&mut state, Some(panel_size)).unwrap();
        assert_close(before_boundary, before - first_delta_lines * line_height);
        assert_eq!(state.processed_top_visual, 0);
        assert_close(state.processed_zoom_anchor_bias_px, 0.0);
        assert!(state.processed_header_scroll_progress < 1.0);

        assert!(apply_processed_panel_vertical_scroll(
            &mut state,
            Some(panel_size),
            0.5,
            40,
        ));
        let after_boundary =
            processed_anchor_page_top_for_state(&mut state, Some(panel_size)).unwrap();
        assert_close(after_boundary, before_boundary - 0.5 * line_height);
        assert_close(state.processed_header_scroll_progress, 1.0);

        assert!(apply_processed_panel_vertical_scroll(
            &mut state,
            Some(panel_size),
            -(first_delta_lines + 0.5),
            40,
        ));
        let restored = processed_anchor_page_top_for_state(&mut state, Some(panel_size)).unwrap();
        assert_close(restored, before);
        assert_close(state.processed_header_scroll_progress, 0.0);
        assert_close(state.processed_zoom_anchor_bias_px, 0.0);
    }

    #[test]
    fn tall_markdown_rows_scroll_by_pixels_instead_of_jumping() {
        let mut state = markdown_state_with_front_matter();
        let panel_size = Vec2::new(1_200.0, 900.0);
        let line_height = scaled_line_height(&state);
        state.processed_header_scroll_progress = 1.0;

        let before = processed_anchor_page_top_for_state(&mut state, Some(panel_size)).unwrap();
        assert!(apply_processed_panel_vertical_scroll(
            &mut state,
            Some(panel_size),
            1.0,
            40,
        ));
        let after = processed_anchor_page_top_for_state(&mut state, Some(panel_size)).unwrap();

        assert_close(after, before - line_height);
        assert_eq!(state.processed_top_visual, 0);
        assert_close(state.processed_zoom_anchor_bias_px, -line_height);
    }

    #[test]
    fn small_scroll_steps_are_continuous_in_both_directions() {
        let mut state = markdown_state_with_front_matter();
        let panel_size = Vec2::new(1_200.0, 900.0);
        let line_height = scaled_line_height(&state);
        let initial = processed_anchor_page_top_for_state(&mut state, Some(panel_size)).unwrap();

        for _ in 0..120 {
            let before = processed_anchor_page_top_for_state(&mut state, Some(panel_size)).unwrap();
            assert!(apply_processed_panel_vertical_scroll(
                &mut state,
                Some(panel_size),
                0.1,
                40,
            ));
            let after = processed_anchor_page_top_for_state(&mut state, Some(panel_size)).unwrap();
            assert_close(after, before - 0.1 * line_height);
        }

        for _ in 0..120 {
            let before = processed_anchor_page_top_for_state(&mut state, Some(panel_size)).unwrap();
            assert!(apply_processed_panel_vertical_scroll(
                &mut state,
                Some(panel_size),
                -0.1,
                40,
            ));
            let after = processed_anchor_page_top_for_state(&mut state, Some(panel_size)).unwrap();
            assert_close(after, before + 0.1 * line_height);
        }

        let restored = processed_anchor_page_top_for_state(&mut state, Some(panel_size)).unwrap();
        assert_close(restored, initial);
    }
}
