fn blink_caret(time: Res<Time>, mut state: ResMut<EditorState>) {
    if state.caret_blink.tick(time.delta()).just_finished() {
        state.caret_visible = !state.caret_visible;
    }
}

fn render_editor(
    body_query: Query<(&PanelBody, &ComputedNode)>,
    mut canvas_query: Query<(&PanelCanvas, &mut UiTransform)>,
    mut text_query: Query<
        (
            &PanelText,
            &mut Text,
            &mut TextFont,
            &mut LineHeight,
            &mut Node,
            &mut UiTransform,
        ),
        (
            Without<StatusText>,
            Without<PanelCaret>,
            Without<PanelPaper>,
            Without<PanelCanvas>,
            Without<ProcessedPaperText>,
            Without<ProcessedPaperLineSpan>,
        ),
    >,
    mut processed_paper_text_query: Query<
        (&ProcessedPaperText, &mut Node, &mut UiTransform),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCaret>,
            Without<PanelCanvas>,
        ),
    >,
    mut processed_span_query: Query<
        (
            &ProcessedPaperLineSpan,
            &mut TextSpan,
            &mut TextFont,
            &mut LineHeight,
            &mut TextColor,
        ),
        Without<PanelText>,
    >,
    text_layout_query: Query<(&PanelText, &TextLayoutInfo)>,
    mut caret_query: Query<
        (&PanelCaret, &mut Node, &mut Visibility, &mut UiTransform),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCanvas>,
        ),
    >,
    mut paper_query: Query<
        (
            &PanelPaper,
            &mut Node,
            &mut Visibility,
            &mut BackgroundColor,
            &mut UiTransform,
        ),
        (
            Without<PanelText>,
            Without<PanelCaret>,
            Without<PanelCanvas>,
        ),
    >,
    mut status_query: Query<&mut Text, (With<StatusText>, Without<PanelText>, Without<PanelCaret>)>,
    fonts: Res<EditorFonts>,
    mut state: ResMut<EditorState>,
) {
    let plain_font_size = scaled_font_size(&state);
    let plain_line_height = state.measured_line_step.max(1.0);
    let plain_char_width = scaled_char_width(&state).max(1.0);
    let plain_origin_y = scaled_text_padding_y(&state);
    let processed_font_size = scaled_font_size(&state);
    let processed_line_height = scaled_line_height(&state).max(1.0);

    let mut plain_inverse_scale = 1.0;
    let mut plain_panel_size = None;
    let mut processed_panel_size = None;

    for (panel, computed) in body_query.iter() {
        let inverse_scale = computed.inverse_scale_factor();
        let logical_size = computed.size() * inverse_scale;
        match panel.kind {
            PanelKind::Plain => {
                plain_inverse_scale = inverse_scale;
                plain_panel_size = Some(logical_size);
            }
            PanelKind::Processed => {
                processed_panel_size = Some(logical_size);
            }
        }
    }
    state.clamp_horizontal_scrolls(plain_panel_size, processed_panel_size);
    let plain_origin_x = scaled_text_padding_x(&state) - state.plain_horizontal_scroll;
    let processed_layout_info =
        processed_page_layout(processed_panel_size.unwrap_or(Vec2::ZERO), &state);
    let processed_geometry = processed_layout_info.geometry;
    let processed_wrap_columns = processed_layout_info.wrap_columns;
    let processed_char_width = scaled_char_width(&state).max(1.0);
    let processed_lines_per_page = processed_layout_info.lines_per_page;
    let processed_spacer_lines = processed_layout_info.spacer_lines;
    let processed_page_step_lines = processed_layout_info.page_step_lines.max(1);
    let visible_lines = viewport_lines(&body_query, state.measured_line_step, plain_origin_y);
    state.clamp_scroll(visible_lines);

    let plain_lines = visible_plain_lines(&state, visible_lines);
    let processed_view_capacity = processed_page_step_lines
        .saturating_mul(PROCESSED_PAPER_CAPACITY)
        .max(1);
    let processed_all_lines = processed_cache_lines(
        &mut state,
        processed_wrap_columns,
        processed_lines_per_page,
        processed_spacer_lines,
    )
    .to_vec();
    let processed_view = build_processed_view(
        &processed_all_lines,
        state.top_line,
        processed_page_step_lines,
        processed_view_capacity,
    );
    let first_visible_page = processed_view.start_index / processed_page_step_lines;
    let processed_anchor_offset_px = processed_view
        .anchor_index
        .saturating_sub(processed_view.start_index) as f32
        * processed_line_height;
    let processed_text_origin_y = processed_geometry.text_top - processed_anchor_offset_px;

    for (_, mut transform) in canvas_query.iter_mut() {
        transform.scale = Vec2::ONE;
        transform.translation = Val2::ZERO;
    }

    for (panel_paper, mut node, mut visibility, mut color, mut transform) in paper_query.iter_mut()
    {
        if panel_paper.kind != PanelKind::Processed {
            *visibility = Visibility::Hidden;
            continue;
        }

        let page_index = first_visible_page.saturating_add(panel_paper.slot);
        let page_start_line = page_index.saturating_mul(processed_page_step_lines);
        let line_delta = page_start_line as isize - processed_view.start_index as isize;
        let page_top_base = processed_geometry.paper_top - processed_anchor_offset_px
            + line_delta as f32 * processed_line_height;
        let page_left = processed_geometry.paper_left - state.processed_horizontal_scroll;
        let page_top = page_top_base;

        node.left = px(page_left);
        node.top = px(page_top);
        node.width = px(processed_geometry.paper_width);
        node.height = px(processed_geometry.paper_height);
        transform.scale = Vec2::ONE;
        transform.translation = Val2::ZERO;
        color.0 = COLOR_PAPER;
        *visibility = Visibility::Visible;
    }

    for (paper_text, mut node, mut transform) in processed_paper_text_query.iter_mut() {
        if paper_text.slot >= PROCESSED_PAPER_CAPACITY {
            continue;
        }

        node.left = px(
            (processed_geometry.text_left - processed_geometry.paper_left)
                - PROCESSED_TEXT_CLIP_BLEED_X,
        );
        node.top = px((processed_geometry.text_top - processed_geometry.paper_top)
            - PROCESSED_TEXT_CLIP_BLEED_Y);
        node.width = px(processed_geometry.text_width + PROCESSED_TEXT_CLIP_BLEED_X * 2.0);
        node.height = px(processed_geometry.text_height + PROCESSED_TEXT_CLIP_BLEED_Y * 2.0);
        node.overflow = Overflow::clip();
        transform.scale = Vec2::ONE;
        transform.translation = Val2::ZERO;
    }

    let plain_view = plain_lines.join("\n");

    for (panel_text, mut text, mut text_font, mut line_height_comp, mut node, mut transform) in
        text_query.iter_mut()
    {
        match panel_text.kind {
            PanelKind::Plain => {
                text_font.font_size = plain_font_size;
                *line_height_comp = LineHeight::Px(plain_line_height);
                **text = plain_view.clone();
                node.left = px(plain_origin_x);
                node.top = px(plain_origin_y);
                node.width = Val::Auto;
                node.height = Val::Auto;
                transform.scale = Vec2::ONE;
                transform.translation = Val2::ZERO;
            }
            PanelKind::Processed => {
                text_font.font_size = processed_font_size;
                *line_height_comp = LineHeight::Px(processed_line_height);
                **text = String::new();
                node.left = px(0.0);
                node.top = px(0.0);
                node.width = px(0.0);
                node.height = px(0.0);
                transform.scale = Vec2::ONE;
                transform.translation = Val2::ZERO;
            }
        }
    }

    apply_processed_styles(
        &mut processed_span_query,
        &state,
        &processed_all_lines,
        first_visible_page,
        processed_page_step_lines,
        processed_lines_per_page,
        &fonts,
        processed_font_size,
        processed_line_height,
    );

    if let Ok(mut status) = status_query.single_mut() {
        **status = state.visible_status();
    }

    let plain_layout = panel_layout_info(&text_layout_query, PanelKind::Plain);
    let processed_layout = None;
    state.measured_line_step = scaled_line_height(&state);

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
                    CARET_X_OFFSET,
                    CARET_WIDTH.max(1.0),
                )
            }
            PanelKind::Processed => {
                let Some((visual_index, display_column, line_text)) =
                    processed_caret_visual(&state, &processed_view)
                else {
                    *visibility = Visibility::Hidden;
                    continue;
                };

                let (processed_origin_x, processed_origin_y) = (
                    processed_geometry.text_left - state.processed_horizontal_scroll,
                    processed_text_origin_y,
                );
                (
                    visual_index,
                    display_column,
                    line_text,
                    processed_layout,
                    1.0,
                    processed_origin_x,
                    processed_origin_y,
                    processed_char_width,
                    processed_line_height,
                    CARET_X_OFFSET,
                    CARET_WIDTH.max(1.0),
                )
            }
        };

        let clamped_display_column = display_column.min(line_text.chars().count());
        let byte_index = char_to_byte_index(line_text, clamped_display_column);
        let caret_x = panel_layout
            .and_then(|layout| {
                caret_x_from_layout(
                    layout,
                    line_offset,
                    line_text,
                    byte_index,
                    panel_inverse_scale,
                    panel_char_width,
                )
            })
            .unwrap_or(clamped_display_column as f32 * panel_char_width);
        let caret_top = panel_layout
            .and_then(|layout| {
                caret_top_from_layout(layout, line_offset, byte_index, panel_inverse_scale)
                    .or_else(|| line_top_from_layout(layout, line_offset, panel_inverse_scale))
            })
            .unwrap_or(line_offset as f32 * panel_line_height);

        let caret_left = origin_x + (caret_x + panel_caret_x_offset).max(0.0);
        let caret_y_offset = CARET_Y_OFFSET_FACTOR * panel_line_height;
        let caret_top = origin_y + (caret_top + caret_y_offset).max(0.0);
        node.left = px(caret_left);
        node.top = px(caret_top);
        node.width = px(panel_caret_width);
        node.height = px(panel_line_height.max(1.0));
        transform.scale = Vec2::ONE;
        transform.translation = Val2::ZERO;
        *visibility = Visibility::Visible;
    }
}

fn viewport_lines(
    body_query: &Query<(&PanelBody, &ComputedNode)>,
    line_step: f32,
    top_padding: f32,
) -> usize {
    let Some((_, computed)) = body_query
        .iter()
        .find(|(panel, _)| panel.kind == PanelKind::Plain)
        .or_else(|| body_query.iter().next())
    else {
        return 24;
    };

    let logical_height = computed.size().y * computed.inverse_scale_factor();
    let step = line_step.max(1.0);
    let usable_height = (logical_height - top_padding).max(step);
    (usable_height / step).floor().max(1.0) as usize
}

fn viewport_lines_from_panels(
    panel_query: &Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
    line_step: f32,
    top_padding: f32,
) -> usize {
    let Some((_, _, computed)) = panel_query
        .iter()
        .find(|(panel, _, _)| panel.kind == PanelKind::Plain)
        .or_else(|| panel_query.iter().next())
    else {
        return 24;
    };

    let logical_height = computed.size().y * computed.inverse_scale_factor();
    let step = line_step.max(1.0);
    let usable_height = (logical_height - top_padding).max(step);
    (usable_height / step).floor().max(1.0) as usize
}

fn visible_plain_lines(state: &EditorState, visible_lines: usize) -> Vec<String> {
    let last = state
        .top_line
        .saturating_add(visible_lines)
        .min(state.document.line_count());

    state
        .document
        .lines()
        .iter()
        .skip(state.top_line)
        .take(last.saturating_sub(state.top_line))
        .cloned()
        .collect()
}

fn ensure_cursor_visible_in_processed_panel(
    state: &mut EditorState,
    processed_panel_size: Option<Vec2>,
    plain_visible_lines: usize,
) {
    let Some(panel_size) = processed_panel_size else {
        return;
    };

    let processed_layout = processed_page_layout(panel_size, state);
    let processed_line_height = scaled_line_height(state).max(1.0);
    let visible_height =
        (panel_size.y - processed_layout.geometry.text_top).max(processed_line_height);
    let processed_visible_lines =
        (visible_height / processed_line_height).floor().max(1.0) as usize;
    let rendered_window = processed_layout
        .page_step_lines
        .max(1)
        .saturating_mul(PROCESSED_PAPER_CAPACITY)
        .max(1);
    let max_window = processed_visible_lines.min(rendered_window).max(1);

    let all_lines = processed_cache_lines(
        state,
        processed_layout.wrap_columns,
        processed_layout.lines_per_page,
        processed_layout.spacer_lines,
    )
    .to_vec();
    if all_lines.is_empty() {
        return;
    }

    let cursor_line = state.cursor.position.line;
    let Some(cursor_visual_index) = first_visual_index_for_source_line(&all_lines, cursor_line)
    else {
        return;
    };

    let max_top_line = state.max_top_line(plain_visible_lines);
    let min_top_for_plain = cursor_line.saturating_sub(plain_visible_lines.saturating_sub(1));
    let max_top_for_plain = cursor_line.min(max_top_line);
    let current_top = state.top_line.min(max_top_line);
    let mut best_top = None;
    let mut best_distance = usize::MAX;

    for candidate_top in min_top_for_plain..=max_top_for_plain {
        let anchor_index = first_visual_index_for_source_line(&all_lines, candidate_top)
            .unwrap_or_else(|| all_lines.len().saturating_sub(1));
        let page_step_lines = processed_layout.page_step_lines.max(1);
        let start_index = (anchor_index / page_step_lines) * page_step_lines;
        let end_index_exclusive = start_index.saturating_add(max_window);

        if cursor_visual_index >= start_index && cursor_visual_index < end_index_exclusive {
            let distance = current_top.abs_diff(candidate_top);
            if distance < best_distance {
                best_distance = distance;
                best_top = Some(candidate_top);
                if distance == 0 {
                    break;
                }
            }
        }
    }

    if let Some(candidate_top) = best_top {
        state.top_line = candidate_top;
        state.clamp_scroll(plain_visible_lines);
    }
}

#[derive(Clone, Debug)]
struct ProcessedVisualLine {
    source_line: usize,
    text: String,
    raw_start_column: usize,
    raw_end_column: usize,
    is_spacer: bool,
}

#[derive(Clone, Debug)]
struct ProcessedSegment {
    start_line: usize,
    end_line_exclusive: usize,
    ends_with_hard_break: bool,
    lines: Vec<ProcessedVisualLine>,
}

#[derive(Clone, Debug)]
struct ProcessedCache {
    wrap_columns: usize,
    lines_per_page: usize,
    spacer_lines: usize,
    segments: Vec<ProcessedSegment>,
    lines: Vec<ProcessedVisualLine>,
    source_line_count: usize,
}

#[derive(Clone, Debug, Default)]
struct ProcessedView {
    start_index: usize,
    anchor_index: usize,
    lines: Vec<ProcessedVisualLine>,
}
