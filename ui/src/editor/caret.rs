pub(crate) const CARET_WIDTH: f32 = 2.0;
pub(crate) const VIM_NORMAL_CARET_WIDTH: f32 = 8.0;
pub(crate) const CARET_X_OFFSET: f32 = -1.0;
// Parley's line metrics already provide the line box top. The old glyph
// metadata was center-based, which needed an upward compensation here.
pub(crate) const CARET_VERTICAL_OFFSET_LINES: f32 = 0.0;

#[derive(Component)]
pub(crate) struct PanelCaret {
    pub(crate) kind: PanelKind,
}

pub(crate) fn blink_caret(time: Res<Time>, mut state: ResMut<EditorState>) {
    if state.caret_blink.tick(time.delta()).just_finished() {
        state.caret_visible = !state.caret_visible;
    }
}

pub(crate) fn caret_vertical_offset(line_height: f32) -> f32 {
    CARET_VERTICAL_OFFSET_LINES * line_height
}

pub(crate) fn caret_width_for_state(state: &EditorState, char_width: f32) -> f32 {
    if state.vim_enabled && state.vim_mode != VimMode::Insert {
        char_width.max(VIM_NORMAL_CARET_WIDTH).max(CARET_WIDTH)
    } else {
        CARET_WIDTH.max(1.0)
    }
}

pub(crate) fn caret_x_offset_for_state(state: &EditorState) -> f32 {
    if state.vim_enabled && state.vim_mode != VimMode::Insert {
        0.0
    } else {
        CARET_X_OFFSET
    }
}

pub(crate) fn processed_caret_line_height(
    state: &EditorState,
    visual_line: &ProcessedVisualLine,
    base_line_height: f32,
) -> f32 {
    let (style, _) = processed_visual_line_style_for_state(state, visual_line);
    (base_line_height * style.line_height_scale).max(1.0)
}

pub(crate) fn render_panel_carets(
    caret_query: &mut Query<
        (&PanelCaret, &mut Node, &mut Visibility, &mut UiTransform),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCanvas>,
            Without<ProcessedChecklistIcon>,
            Without<ProcessedImageBlockNode>,
        ),
    >,
    state: &EditorState,
    visible_lines: usize,
    plain_lines: &[String],
    plain_layout: Option<&ComputedTextBlock>,
    plain_inverse_scale: f32,
    plain_origin_x: f32,
    plain_origin_y: f32,
    plain_char_width: f32,
    plain_line_height: f32,
    processed_view: &ProcessedView,
    first_visible_page: usize,
    processed_page_step_lines: usize,
    processed_lines_per_page: usize,
    processed_text_layout_query: &Query<
        (&ProcessedPaperText, &ComputedTextBlock, &ComputedNode),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCaret>,
            Without<PanelCanvas>,
        ),
    >,
    processed_geometry: &ProcessedPageGeometry,
    processed_page_step_pixels: f32,
    processed_anchor_offset_px: f32,
    processed_zoom_bias_px: f32,
    processed_char_width: f32,
    processed_line_height: f32,
) {
    for (panel_caret, mut node, mut visibility, mut transform) in caret_query.iter_mut() {
        if !state.caret_visible {
            *visibility = Visibility::Hidden;
            continue;
        }

        let (
            line_offset,
            display_column,
            line_text,
            panel_layout,
            panel_inverse_scale,
            origin_x,
            origin_y,
            panel_char_width,
            panel_line_height,
            panel_caret_x_offset,
            panel_caret_width,
            fallback_caret_height,
            clamp_display_column,
            clamp_local_position_to_origin,
        ) = match panel_caret.kind {
            PanelKind::Plain => {
                let in_view = state.cursor.position.line >= state.top_line
                    && state.cursor.position.line < state.top_line + visible_lines;
                if !in_view {
                    *visibility = Visibility::Hidden;
                    continue;
                }

                let line_offset = state.cursor.position.line - state.top_line;
                let line_text = plain_lines
                    .get(line_offset)
                    .map_or("", |line| line.as_str());
                (
                    line_offset,
                    state.cursor.position.column,
                    line_text,
                    plain_layout,
                    plain_inverse_scale,
                    plain_origin_x,
                    plain_origin_y,
                    plain_char_width,
                    plain_line_height,
                    caret_x_offset_for_state(state),
                    caret_width_for_state(state, plain_char_width),
                    plain_line_height,
                    true,
                    true,
                )
            }
            PanelKind::Processed => {
                let Some((visual_index, display_column, visual_line)) =
                    processed_caret_visual(state, processed_view)
                else {
                    *visibility = Visibility::Hidden;
                    continue;
                };

                let global_index = processed_view.start_index.saturating_add(visual_index);
                let page_index = global_index / processed_page_step_lines;
                let line_in_page = global_index % processed_page_step_lines;
                if line_in_page >= processed_lines_per_page {
                    *visibility = Visibility::Hidden;
                    continue;
                }
                let slot = page_index.saturating_sub(first_visible_page);
                let (processed_layout, processed_inverse_scale) = processed_text_layout_query
                    .iter()
                    .find(|(paper_text, _, _)| {
                        paper_text.slot == slot && paper_text.line_offset == line_in_page
                    })
                    .map_or((None, 1.0), |(_, text_block, computed)| {
                        (Some(text_block), computed.inverse_scale_factor())
                    });

                let page_text_top = processed_content_text_top_for_slot(
                    state,
                    &processed_view.lines,
                    processed_geometry,
                    slot,
                    processed_page_step_lines,
                    processed_line_height,
                    processed_page_step_pixels,
                    processed_anchor_offset_px,
                ) + processed_zoom_bias_px;
                let page_start_in_view = slot.saturating_mul(processed_page_step_lines);
                let line_top = processed_visual_line_top_units(
                    state,
                    &processed_view.lines,
                    page_start_in_view,
                    line_in_page,
                ) * processed_line_height;
                let (processed_origin_x, processed_origin_y) = (
                    processed_geometry.text_left - state.processed_horizontal_scroll,
                    page_text_top + line_top,
                );
                (
                    0,
                    display_column,
                    visual_line.text.as_str(),
                    processed_layout,
                    processed_inverse_scale,
                    processed_origin_x,
                    processed_origin_y,
                    processed_char_width,
                    processed_line_height,
                    caret_x_offset_for_state(state),
                    caret_width_for_state(state, processed_char_width),
                    processed_caret_line_height(state, visual_line, processed_line_height),
                    true,
                    true,
                )
            }
        };

        let display_column = if clamp_display_column {
            display_column.min(line_text.chars().count())
        } else {
            display_column
        };
        let byte_index = char_to_byte_index(line_text, display_column);
        let caret_x = panel_layout
            .and_then(|text_block| {
                caret_x_from_layout(
                    text_block,
                    line_offset,
                    line_text,
                    byte_index,
                    panel_inverse_scale,
                    panel_char_width,
                )
            })
            .unwrap_or(display_column as f32 * panel_char_width);
        let caret_top = panel_layout
            .and_then(|text_block| {
                line_top_from_layout(text_block, line_offset, panel_inverse_scale)
            })
            .unwrap_or(line_offset as f32 * panel_line_height);
        let caret_height = panel_layout
            .and_then(|text_block| {
                line_height_from_layout(text_block, line_offset, panel_inverse_scale)
            })
            .unwrap_or(fallback_caret_height)
            .max(1.0);

        let local_caret_left = if clamp_local_position_to_origin {
            (caret_x + panel_caret_x_offset).max(0.0)
        } else {
            caret_x + panel_caret_x_offset
        };
        let caret_left = origin_x + local_caret_left;
        let caret_y_offset = caret_vertical_offset(caret_height);
        let local_caret_top = if clamp_local_position_to_origin {
            (caret_top + caret_y_offset).max(0.0)
        } else {
            caret_top + caret_y_offset
        };
        let caret_top = origin_y + local_caret_top;
        node.left = px(caret_left);
        node.top = px(caret_top);
        node.width = px(panel_caret_width);
        node.height = px(caret_height);
        transform.scale = Vec2::ONE;
        transform.translation = Val2::ZERO;
        *visibility = Visibility::Visible;
    }
}
#[allow(unused_imports)]
use super::*;
