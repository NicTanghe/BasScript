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
            Without<ProcessedImageBlockNode>,
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
            Without<ProcessedImageBlockNode>,
        ),
    >,
    mut processed_checklist_icon_query: Query<
        (
            &ProcessedChecklistIcon,
            &mut ImageNode,
            &mut Node,
            &mut Visibility,
        ),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCaret>,
            Without<PanelCanvas>,
            Without<ProcessedPaperText>,
            Without<ProcessedPaperLineSpan>,
            Without<ProcessedImageBlockNode>,
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
    text_layout_query: Query<(&PanelText, &ComputedTextBlock)>,
    processed_text_layout_query: Query<
        (&ProcessedPaperText, &ComputedTextBlock, &ComputedNode),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCaret>,
            Without<PanelCanvas>,
        ),
    >,
    mut caret_query: Query<
        (&PanelCaret, &mut Node, &mut Visibility, &mut UiTransform),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCanvas>,
            Without<ProcessedChecklistIcon>,
            Without<ProcessedImageBlockNode>,
        ),
    >,
    mut selection_rect_query: Query<
        (
            &PanelSelectionRect,
            &mut Node,
            &mut BackgroundColor,
            &mut Visibility,
        ),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCaret>,
            Without<PanelCanvas>,
            Without<ProcessedPaperText>,
            Without<ProcessedChecklistIcon>,
            Without<ProcessedImageBlockNode>,
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
            Without<ProcessedImageBlockNode>,
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

    if state.document_format == DocumentFormat::Canvas {
        maybe_center_canvas_view_after_layout(&body_query, &mut state);
        for (_, mut text, _, _, mut node, mut transform) in text_query.iter_mut() {
            **text = String::new();
            node.left = px(0.0);
            node.top = px(0.0);
            node.width = px(0.0);
            node.height = px(0.0);
            transform.scale = Vec2::ONE;
            transform.translation = Val2::ZERO;
        }
        for (_, mut node, mut transform) in processed_paper_text_query.iter_mut() {
            node.width = px(0.0);
            node.height = px(0.0);
            transform.scale = Vec2::ONE;
            transform.translation = Val2::ZERO;
        }
        for (_, mut image_node, mut node, mut visibility) in
            processed_checklist_icon_query.iter_mut()
        {
            image_node.color = Color::WHITE;
            node.width = px(0.0);
            node.height = px(0.0);
            *visibility = Visibility::Hidden;
        }
        for (_, mut text_span, _, _, mut text_color) in processed_span_query.iter_mut() {
            **text_span = String::new();
            text_color.0 = Color::NONE;
        }
        for (_, mut node, mut visibility, mut transform) in caret_query.iter_mut() {
            node.width = px(0.0);
            node.height = px(0.0);
            transform.scale = Vec2::ONE;
            transform.translation = Val2::ZERO;
            *visibility = Visibility::Hidden;
        }
        for (_, mut node, _, mut visibility) in selection_rect_query.iter_mut() {
            node.width = px(0.0);
            node.height = px(0.0);
            *visibility = Visibility::Hidden;
        }
        for (_, _, mut visibility, _, _) in paper_query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        for (_, mut transform) in canvas_query.iter_mut() {
            transform.scale = Vec2::ONE;
            transform.translation = Val2::ZERO;
        }
        if let Ok(mut status) = status_query.single_mut() {
            **status = state.visible_status();
        }
        return;
    }

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
    let processed_total_pages =
        processed_page_count_for_lines(&processed_all_lines, processed_page_step_lines);
    let processed_anchor_offset_px = processed_anchor_scroll_offset_px_from_lines(
        &state,
        &processed_all_lines,
        processed_view.anchor_index,
        processed_page_step_lines,
        processed_line_height,
    );
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

        let page_index = first_visible_page.saturating_add(panel_paper.slot);
        if page_index >= processed_total_pages {
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

        let page_index = first_visible_page.saturating_add(paper_text.slot);
        let page_start = page_index.saturating_mul(processed_page_step_lines);
        let global_index = page_start.saturating_add(paper_text.line_offset);
        let Some(visual_line) = processed_all_lines.get(global_index) else {
            if node.width != px(0.0) {
                node.width = px(0.0);
            }
            if node.height != px(0.0) {
                node.height = px(0.0);
            }
            continue;
        };
        if page_index >= processed_total_pages
            || paper_text.line_offset >= processed_lines_per_page
        {
            if node.width != px(0.0) {
                node.width = px(0.0);
            }
            if node.height != px(0.0) {
                node.height = px(0.0);
            }
            continue;
        }

        let line_top_units = processed_visual_line_top_units(
            &state,
            &processed_all_lines,
            page_start,
            paper_text.line_offset,
        );
        let line_height_units = processed_visual_line_height_units(&state, visual_line);
        let next_left = px(processed_geometry.text_left - processed_geometry.paper_left);
        let next_top = px(
            processed_geometry.text_top - processed_geometry.paper_top
                + line_top_units * processed_line_height,
        );
        let next_width = px(processed_geometry.text_width);
        let next_height = px(line_height_units * processed_line_height);
        if node.left != next_left {
            node.left = next_left;
        }
        if node.top != next_top {
            node.top = next_top;
        }
        if node.width != next_width {
            node.width = next_width;
        }
        if node.height != next_height {
            node.height = next_height;
        }
        if node.overflow != Overflow::visible() {
            node.overflow = Overflow::visible();
        }
        if transform.scale != Vec2::ONE {
            transform.scale = Vec2::ONE;
        }
        if transform.translation != Val2::ZERO {
            transform.translation = Val2::ZERO;
        }
    }

    let text_left_in_paper = processed_geometry.text_left - processed_geometry.paper_left;
    let text_top_in_paper = processed_geometry.text_top - processed_geometry.paper_top;
    let checklist_icon_size = (processed_line_height * 0.72).clamp(8.0, 16.0);
    let checklist_icon_gap = (processed_line_height * 0.20).clamp(2.0, 4.0);

    for (icon, mut image_node, mut node, mut visibility) in
        processed_checklist_icon_query.iter_mut()
    {
        if icon.slot >= PROCESSED_PAPER_CAPACITY {
            *visibility = Visibility::Hidden;
            continue;
        }

        let page_index = first_visible_page.saturating_add(icon.slot);
        let line_offset = icon
            .line_offset
            .min(processed_page_step_lines.saturating_sub(1));
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
        let line_top_units = processed_visual_line_top_units(
            &state,
            &processed_all_lines,
            page_start,
            line_offset,
        );
        node.left = px((text_left_in_paper - checklist_icon_size - checklist_icon_gap).max(0.0));
        node.top = px(text_top_in_paper
            + line_top_units * processed_line_height
            + ((processed_line_height - checklist_icon_size) * 0.5).max(0.0));
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
                )
                .into();
                text_font.font_size = FontSize::Px(plain_font_size);
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
                text_font.font_size = FontSize::Px(processed_font_size);
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
    render_selection_rects(
        &mut selection_rect_query,
        &state,
        &plain_lines,
        plain_layout,
        plain_inverse_scale,
        plain_origin_x,
        plain_origin_y,
        plain_char_width,
        plain_line_height,
        &processed_view,
        first_visible_page,
        processed_page_step_lines,
        processed_lines_per_page,
        &processed_text_layout_query,
        &processed_geometry,
        processed_page_step_pixels,
        processed_anchor_offset_px,
        processed_zoom_bias_px,
        processed_char_width,
        processed_line_height,
    );

    render_panel_carets(
        &mut caret_query,
        &state,
        visible_lines,
        &plain_lines,
        plain_layout,
        plain_inverse_scale,
        plain_origin_x,
        plain_origin_y,
        plain_char_width,
        plain_line_height,
        &processed_view,
        first_visible_page,
        processed_page_step_lines,
        processed_lines_per_page,
        &processed_text_layout_query,
        &processed_geometry,
        processed_page_step_pixels,
        processed_anchor_offset_px,
        processed_zoom_bias_px,
        processed_char_width,
        processed_line_height,
    );
}

fn render_processed_images(
    body_query: Query<(&PanelBody, &ComputedNode)>,
    mut processed_image_query: Query<
        (
            &ProcessedImageBlockNode,
            &mut ImageNode,
            &mut Node,
            &mut Visibility,
        ),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCaret>,
            Without<PanelCanvas>,
            Without<ProcessedPaperText>,
            Without<ProcessedPaperLineSpan>,
            Without<ProcessedChecklistIcon>,
        ),
    >,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<EditorImageCache>,
    mut state: ResMut<EditorState>,
) {
    if state.document_format == DocumentFormat::Canvas {
        for (_, _, _, mut visibility) in processed_image_query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let processed_panel_size = body_query
        .iter()
        .find(|(panel, _)| panel.kind == PanelKind::Processed)
        .map(|(_, computed)| computed.size() * computed.inverse_scale_factor());

    let Some(processed_panel_size) = processed_panel_size
        .filter(|_| state.panel_visible(PanelKind::Processed))
        .filter(|size| size.x > 1.0 && size.y > 1.0)
    else {
        for (_, _, _, mut visibility) in processed_image_query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    let processed_layout_info = processed_page_layout(processed_panel_size, &state);
    let processed_geometry = processed_layout_info.geometry;
    let processed_wrap_columns = processed_layout_info.wrap_columns;
    let processed_lines_per_page = processed_layout_info.lines_per_page;
    let processed_spacer_lines = processed_layout_info.spacer_lines;
    let processed_page_step_lines = processed_layout_info.page_step_lines.max(1);
    let processed_line_height = scaled_line_height(&state).max(1.0);
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

    let processed_view_capacity = processed_page_step_lines
        .saturating_mul(PROCESSED_PAPER_CAPACITY)
        .max(1);
    let processed_view = build_processed_view(
        &processed_all_lines,
        state.processed_top_visual,
        processed_page_step_lines,
        processed_view_capacity,
    );
    let first_visible_page = processed_view.start_index / processed_page_step_lines;
    let text_left_in_paper = processed_geometry.text_left - processed_geometry.paper_left;
    let text_top_in_paper = processed_geometry.text_top - processed_geometry.paper_top;

    for (image_block_node, mut image_node, mut node, mut visibility) in
        processed_image_query.iter_mut()
    {
        if image_block_node.slot >= PROCESSED_PAPER_CAPACITY {
            *visibility = Visibility::Hidden;
            continue;
        }

        let page_index = first_visible_page.saturating_add(image_block_node.slot);
        let line_offset = image_block_node
            .line_offset
            .min(processed_page_step_lines.saturating_sub(1));
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
        let Some(image_block) = visual_line.image_block.as_ref() else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let reserved_height =
            image_block.reserved_lines as f32 * processed_line_height - PROCESSED_IMAGE_BLOCK_GAP;
        if reserved_height <= 1.0 {
            *visibility = Visibility::Hidden;
            continue;
        }

        let lookup = processed_image_lookup(
            &mut image_cache,
            &state,
            &image_block.target,
            &asset_server,
            &mut images,
        );
        let image_size = match lookup {
            ProcessedImageLookup::Loaded { handle, size } => {
                image_node.image = handle;
                image_node.color = Color::WHITE;
                size
            }
            ProcessedImageLookup::Failed => {
                *image_node = ImageNode::solid_color(COLOR_IMAGE_PLACEHOLDER);
                UVec2::new(16, 9)
            }
        };

        let image_width = image_size.x.max(1) as f32;
        let image_height = image_size.y.max(1) as f32;
        let max_width = processed_geometry.text_width.max(1.0);
        let natural_height = max_width * image_height / image_width;
        let display_height = natural_height.min(reserved_height).max(1.0);
        let display_width = (display_height * image_width / image_height)
            .min(max_width)
            .max(1.0);
        let left = text_left_in_paper + (max_width - display_width) * 0.5;
        let line_top_units = processed_visual_line_top_units(
            &state,
            &processed_all_lines,
            page_start,
            line_offset,
        );
        let top = text_top_in_paper
            + line_top_units * processed_line_height
            + ((reserved_height - display_height) * 0.5).max(0.0);

        node.left = px(left);
        node.top = px(top);
        node.width = px(display_width);
        node.height = px(display_height);
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
struct ProcessedVisualFragment {
    text: String,
    is_link: bool,
    link_target: Option<String>,
    inline_style: InlineTextStyle,
}

#[derive(Clone, Debug)]
struct ProcessedVisualLine {
    source_line: usize,
    text: String,
    fragments: Vec<ProcessedVisualFragment>,
    display_to_raw: Vec<usize>,
    raw_start_column: usize,
    raw_end_column: usize,
    markdown_checklist_checked: Option<bool>,
    image_block: Option<ProcessedImageBlock>,
    render_override: Option<ProcessedLineRenderOverride>,
    is_spacer: bool,
}

#[derive(Clone, Debug)]
struct ProcessedImageBlock {
    target: String,
    reserved_lines: usize,
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

#[derive(Clone, Debug)]
struct ProcessedLineRenderOverride {
    kind: LineKind,
    markdown_heading_level: Option<u8>,
}

#[derive(Clone, Copy, Debug)]
struct LineRenderStyle {
    font_variant: FontVariant,
    color: Color,
    font_scale: f32,
    line_height_scale: f32,
}

impl LineRenderStyle {
    const fn new(
        font_variant: FontVariant,
        color: Color,
        font_scale: f32,
        line_height_scale: f32,
    ) -> Self {
        Self {
            font_variant,
            color,
            font_scale,
            line_height_scale,
        }
    }
}

fn transparent_line_render_style() -> LineRenderStyle {
    LineRenderStyle::new(
        FontVariant::Regular,
        Color::srgba(0.0, 0.0, 0.0, 0.0),
        1.0,
        1.0,
    )
}

fn default_line_render_style() -> LineRenderStyle {
    LineRenderStyle::new(FontVariant::Regular, COLOR_ACTION, 1.0, 1.0)
}

fn processed_line_style(parsed_line: &ParsedLine) -> LineRenderStyle {
    processed_line_style_for_kind(&parsed_line.kind, parsed_line.markdown_heading_level)
}

fn processed_line_style_for_kind(
    kind: &LineKind,
    markdown_heading_level: Option<u8>,
) -> LineRenderStyle {
    fountain_line_style(kind)
        .or_else(|| markdown_line_style(kind, markdown_heading_level))
        .unwrap_or_else(default_line_render_style)
}

fn processed_visual_line_style_for_state(
    state: &EditorState,
    visual_line: &ProcessedVisualLine,
) -> (LineRenderStyle, bool) {
    let raw_current_line_mode_active = state.display_mode == DisplayMode::ProcessedRawCurrentLine
        && visual_line.source_line == state.cursor.position.line;

    if visual_line.is_spacer {
        (transparent_line_render_style(), false)
    } else if let Some(render_override) = visual_line.render_override.as_ref() {
        (
            processed_line_style_for_kind(
                &render_override.kind,
                render_override.markdown_heading_level,
            ),
            true,
        )
    } else if raw_current_line_mode_active {
        (default_line_render_style(), false)
    } else if let Some(parsed_line) = state.parsed.get(visual_line.source_line) {
        (processed_line_style(parsed_line), true)
    } else {
        (default_line_render_style(), false)
    }
}

fn font_variant_for_processed_fragment(
    base: FontVariant,
    fragment: &ProcessedVisualFragment,
    format: DocumentFormat,
) -> FontVariant {
    let mut style = fragment.inline_style;
    if fragment.is_link && format == DocumentFormat::Fountain {
        style.bold = true;
    }

    apply_inline_style_to_font_variant(base, style)
}

fn font_for_variant_with_format(
    fonts: &EditorFonts,
    variant: FontVariant,
    format: DocumentFormat,
) -> Handle<Font> {
    match format {
        DocumentFormat::Markdown | DocumentFormat::Canvas => match variant {
            FontVariant::Regular => fonts.markdown_regular.clone(),
            FontVariant::Bold => fonts.markdown_bold.clone(),
            FontVariant::Italic => fonts.markdown_italic.clone(),
            FontVariant::BoldItalic => fonts.markdown_bold_italic.clone(),
        },
        DocumentFormat::Fountain => match variant {
            FontVariant::Regular => fonts.regular.clone(),
            FontVariant::Bold => fonts.bold.clone(),
            FontVariant::Italic => fonts.italic.clone(),
            FontVariant::BoldItalic => fonts.bold_italic.clone(),
        },
    }
}

fn processed_image_lookup(
    cache: &mut EditorImageCache,
    state: &EditorState,
    target: &str,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
) -> ProcessedImageLookup {
    let trimmed = target.trim();
    if is_remote_image_target(trimmed) {
        return ProcessedImageLookup::Loaded {
            handle: asset_server.load(trimmed.to_owned()),
            size: UVec2::new(640, 360),
        };
    }

    let resolved = match resolve_processed_image_path(state, target) {
        Ok(path) => path,
        Err(_) => return ProcessedImageLookup::Failed,
    };
    let modified = fs::metadata(&resolved)
        .and_then(|metadata| metadata.modified())
        .ok();

    if let Some(cached) = cache.entries.get(&resolved) {
        if cached.modified == modified {
            return match &cached.result {
                CachedProcessedImageResult::Loaded { handle, size } => {
                    ProcessedImageLookup::Loaded {
                        handle: handle.clone(),
                        size: *size,
                    }
                }
                CachedProcessedImageResult::Failed => ProcessedImageLookup::Failed,
            };
        }
    }

    let result = load_processed_image(&resolved, images);
    cache.entries.insert(
        resolved,
        CachedProcessedImage {
            modified,
            result: result.clone(),
        },
    );

    match result {
        CachedProcessedImageResult::Loaded { handle, size } => {
            ProcessedImageLookup::Loaded { handle, size }
        }
        CachedProcessedImageResult::Failed => ProcessedImageLookup::Failed,
    }
}

fn resolve_processed_image_path(state: &EditorState, target: &str) -> Result<PathBuf, String> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return Err("empty image path".to_owned());
    }
    if is_remote_image_target(trimmed) || trimmed.starts_with("data:") {
        return Err(format!("remote image targets are not supported: {trimmed}"));
    }

    let target_path = Path::new(trimmed);
    if target_path.is_absolute() {
        return Ok(canonicalize_if_possible(target_path.to_path_buf()));
    }

    let document_candidate = state
        .paths
        .save_path
        .parent()
        .map(|parent| parent.join(target_path));
    if let Some(candidate) = document_candidate.as_ref().filter(|path| path.exists()) {
        return Ok(canonicalize_if_possible(candidate.clone()));
    }

    if let Some(root) = state.workspace_root.as_ref() {
        let workspace_candidate = root.join(target_path);
        if workspace_candidate.exists() || document_candidate.is_none() {
            return Ok(canonicalize_if_possible(workspace_candidate));
        }
    }

    for ancestor in state.paths.save_path.ancestors().skip(1) {
        let ancestor_candidate = ancestor.join(target_path);
        if ancestor_candidate.exists() {
            return Ok(canonicalize_if_possible(ancestor_candidate));
        }
    }

    for suffix in relative_path_suffixes(target_path) {
        if let Some(root) = state.workspace_root.as_ref() {
            let workspace_candidate = root.join(&suffix);
            if workspace_candidate.exists() {
                return Ok(canonicalize_if_possible(workspace_candidate));
            }
        }

        for ancestor in state.paths.save_path.ancestors().skip(1) {
            let ancestor_candidate = ancestor.join(&suffix);
            if ancestor_candidate.exists() {
                return Ok(canonicalize_if_possible(ancestor_candidate));
            }
        }
    }

    document_candidate
        .map(canonicalize_if_possible)
        .ok_or_else(|| format!("could not resolve image path: {trimmed}"))
}

fn relative_path_suffixes(path: &Path) -> Vec<PathBuf> {
    let components = path
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(part.to_owned()),
            _ => None,
        })
        .collect::<Vec<_>>();

    (1..components.len())
        .map(|start| {
            components[start..]
                .iter()
                .fold(PathBuf::new(), |mut suffix, part| {
                    suffix.push(part);
                    suffix
                })
        })
        .collect()
}

fn canonicalize_if_possible(path: PathBuf) -> PathBuf {
    fs::canonicalize(&path).unwrap_or(path)
}

fn is_remote_image_target(target: &str) -> bool {
    target.starts_with("http://") || target.starts_with("https://")
}

fn load_processed_image(path: &Path, images: &mut Assets<Image>) -> CachedProcessedImageResult {
    if !path.exists() {
        return CachedProcessedImageResult::Failed;
    }

    let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
        return CachedProcessedImageResult::Failed;
    };
    if !matches!(
        extension.to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "bmp"
    ) {
        return CachedProcessedImageResult::Failed;
    }

    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            let _ = error;
            return CachedProcessedImageResult::Failed;
        }
    };

    let image = match Image::from_buffer(
        &bytes,
        ImageType::Extension(extension),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::linear(),
        RenderAssetUsages::default(),
    ) {
        Ok(image) => image,
        Err(error) => {
            let _ = error;
            return CachedProcessedImageResult::Failed;
        }
    };

    let size = UVec2::new(
        image.texture_descriptor.size.width,
        image.texture_descriptor.size.height,
    );
    let handle = images.add(image);
    CachedProcessedImageResult::Loaded { handle, size }
}
