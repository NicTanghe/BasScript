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
            Without<ProcessedChecklistIcon>,
        ),
    >,
    mut processed_paper_text_query: Query<
        (&ProcessedPaperText, &mut Node, &mut UiTransform),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCaret>,
            Without<PanelCanvas>,
            Without<ProcessedChecklistIcon>,
        ),
    >,
    mut processed_checklist_icon_query: Query<
        (&ProcessedChecklistIcon, &mut ImageNode, &mut Node, &mut Visibility),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCaret>,
            Without<PanelCanvas>,
            Without<ProcessedPaperText>,
            Without<ProcessedPaperLineSpan>,
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
    processed_text_layout_query: Query<
        (&ProcessedPaperText, &TextLayoutInfo, &ComputedNode),
        (Without<PanelText>, Without<PanelPaper>, Without<PanelCaret>, Without<PanelCanvas>),
    >,
    mut caret_query: Query<
        (&PanelCaret, &mut Node, &mut Visibility, &mut UiTransform),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCanvas>,
            Without<ProcessedChecklistIcon>,
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
            Without<ProcessedChecklistIcon>,
        ),
    >,
    mut status_query: Query<&mut Text, (With<StatusText>, Without<PanelText>, Without<PanelCaret>)>,
    fonts: Res<EditorFonts>,
    checklist_icons: Res<ChecklistIcons>,
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
    let visible_lines = viewport_lines(
        &body_query,
        state.display_mode,
        state.measured_line_step,
        plain_origin_y,
    );
    state.clamp_scroll(visible_lines);
    state.clamp_processed_top_line();

    let plain_lines = visible_plain_lines(&state, visible_lines);
    let processed_view_capacity = processed_page_step_lines
        .saturating_mul(PROCESSED_PAPER_CAPACITY)
        .max(1);
    let processed_all_lines = processed_display_lines(
        &mut state,
        processed_wrap_columns,
        processed_lines_per_page,
        processed_spacer_lines,
    );
    if processed_all_lines.is_empty() {
        state.processed_top_visual = 0;
    } else {
        state.processed_top_visual = state
            .processed_top_visual
            .min(processed_all_lines.len().saturating_sub(1));
    }
    let processed_view = build_processed_view(
        &processed_all_lines,
        state.processed_top_visual,
        processed_page_step_lines,
        processed_view_capacity,
    );
    let first_visible_page = processed_view.start_index / processed_page_step_lines;
    let anchor_line_in_page =
        processed_anchor_line_in_page(&processed_view, processed_page_step_lines);
    let processed_anchor_offset_px =
        processed_anchor_scroll_offset_px(anchor_line_in_page, processed_line_height);
    let processed_page_step_pixels = processed_page_step_px(&processed_geometry, state.zoom);
    let processed_zoom_bias_px = state.processed_zoom_anchor_bias_px;

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

        let page_top = processed_page_top_for_slot(
            &processed_geometry,
            panel_paper.slot,
            processed_page_step_pixels,
            processed_anchor_offset_px,
        ) + processed_zoom_bias_px;
        let page_left = processed_geometry.paper_left - state.processed_horizontal_scroll;

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

        node.left = px(processed_geometry.text_left - processed_geometry.paper_left);
        node.top = px(processed_geometry.text_top - processed_geometry.paper_top);
        node.width = px(processed_geometry.text_width);
        node.height = px(processed_geometry.text_height);
        node.overflow = Overflow::visible();
        transform.scale = Vec2::ONE;
        transform.translation = Val2::ZERO;
    }

    let text_left_in_paper = processed_geometry.text_left - processed_geometry.paper_left;
    let text_top_in_paper = processed_geometry.text_top - processed_geometry.paper_top;
    let checklist_icon_size = (processed_line_height * 0.72).clamp(8.0, 16.0);
    let checklist_icon_gap = (processed_line_height * 0.20).clamp(2.0, 4.0);

    for (icon, mut image_node, mut node, mut visibility) in processed_checklist_icon_query.iter_mut()
    {
        if icon.slot >= PROCESSED_PAPER_CAPACITY {
            *visibility = Visibility::Hidden;
            continue;
        }

        let page_index = first_visible_page.saturating_add(icon.slot);
        let line_offset = icon.line_offset.min(processed_page_step_lines.saturating_sub(1));
        if line_offset >= processed_lines_per_page {
            *visibility = Visibility::Hidden;
            continue;
        }

        let page_start = page_index.saturating_mul(processed_page_step_lines);
        let global_index = page_start.saturating_add(line_offset);
        let Some(visual_line) = processed_all_lines.get(global_index) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let Some(checked) = visual_line.markdown_checklist_checked else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if visual_line.is_spacer {
            *visibility = Visibility::Hidden;
            continue;
        }

        image_node.image = if checked {
            checklist_icons.checked.clone()
        } else {
            checklist_icons.unchecked.clone()
        };
        node.left = px((text_left_in_paper - checklist_icon_size - checklist_icon_gap).max(0.0));
        node.top = px(
            text_top_in_paper
                + line_offset as f32 * processed_line_height
                + ((processed_line_height - checklist_icon_size) * 0.5).max(0.0),
        );
        node.width = px(checklist_icon_size);
        node.height = px(checklist_icon_size);
        *visibility = Visibility::Visible;
    }

    let plain_view = plain_lines.join("\n");

    for (panel_text, mut text, mut text_font, mut line_height_comp, mut node, mut transform) in
        text_query.iter_mut()
    {
        match panel_text.kind {
            PanelKind::Plain => {
                text_font.font = font_for_variant_with_format(
                    &fonts,
                    FontVariant::Regular,
                    state.document_format,
                );
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
                    CARET_X_OFFSET,
                    CARET_WIDTH.max(1.0),
                    true,
                    true,
                )
            }
            PanelKind::Processed => {
                let Some((visual_index, display_column, line_text)) =
                    processed_caret_visual(&state, &processed_view)
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
                    .find(|(paper_text, _, _)| paper_text.slot == slot)
                    .map_or((None, 1.0), |(_, layout, computed)| {
                        (Some(layout), computed.inverse_scale_factor())
                    });

                let page_text_top = processed_text_top_for_slot(
                    &processed_geometry,
                    slot,
                    processed_page_step_pixels,
                    processed_anchor_offset_px,
                ) + processed_zoom_bias_px;
                let (processed_origin_x, processed_origin_y) = (
                    processed_geometry.text_left - state.processed_horizontal_scroll,
                    page_text_top,
                );
                (
                    line_in_page,
                    display_column,
                    line_text,
                    processed_layout,
                    processed_inverse_scale,
                    processed_origin_x,
                    processed_origin_y,
                    processed_char_width,
                    processed_line_height,
                    CARET_X_OFFSET,
                    CARET_WIDTH.max(1.0),
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
            .unwrap_or(display_column as f32 * panel_char_width);
        let caret_top = panel_layout
            .and_then(|layout| {
                caret_top_from_layout(layout, line_offset, byte_index, panel_inverse_scale)
                    .or_else(|| line_top_from_layout(layout, line_offset, panel_inverse_scale))
            })
            .unwrap_or(line_offset as f32 * panel_line_height);

        let local_caret_left = if clamp_local_position_to_origin {
            (caret_x + panel_caret_x_offset).max(0.0)
        } else {
            caret_x + panel_caret_x_offset
        };
        let caret_left = origin_x + local_caret_left;
        let caret_y_offset = CARET_Y_OFFSET_FACTOR * panel_line_height;
        let local_caret_top = if clamp_local_position_to_origin {
            (caret_top + caret_y_offset).max(0.0)
        } else {
            caret_top + caret_y_offset
        };
        let caret_top = origin_y + local_caret_top;
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
    display_mode: DisplayMode,
    line_step: f32,
    top_padding: f32,
) -> usize {
    let preferred_panel = match display_mode {
        DisplayMode::Processed | DisplayMode::ProcessedRawCurrentLine => PanelKind::Processed,
        DisplayMode::Split | DisplayMode::Plain => PanelKind::Plain,
    };
    let Some((_, computed)) = body_query
        .iter()
        .find(|(panel, _)| panel.kind == preferred_panel)
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
    display_mode: DisplayMode,
    line_step: f32,
    top_padding: f32,
) -> usize {
    let preferred_panel = match display_mode {
        DisplayMode::Processed | DisplayMode::ProcessedRawCurrentLine => PanelKind::Processed,
        DisplayMode::Split | DisplayMode::Plain => PanelKind::Plain,
    };
    let Some((_, _, computed)) = panel_query
        .iter()
        .find(|(panel, _, _)| panel.kind == preferred_panel)
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

#[derive(Clone, Debug)]
struct ProcessedVisualLine {
    source_line: usize,
    text: String,
    display_indent_width: usize,
    raw_start_column: usize,
    raw_end_column: usize,
    markdown_checklist_checked: Option<bool>,
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
