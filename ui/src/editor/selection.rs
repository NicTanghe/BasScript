#[derive(Resource, Default, Clone, Copy, Debug)]
struct MouseSelectionState {
    active: bool,
    extend_from_existing: bool,
    dragged: bool,
}

fn setup_selection_rects(
    mut commands: Commands,
    selection_layer_query: Query<(Entity, &PanelSelectionLayer)>,
) {
    for (entity, selection_layer) in selection_layer_query.iter() {
        let kind = selection_layer.kind;
        commands.entity(entity).with_children(|parent| {
            for index in 0..SELECTION_RECT_CAPACITY {
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        width: px(0.0),
                        height: px(0.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                    Visibility::Hidden,
                    ZIndex(1),
                    PanelSelectionRect { kind, index },
                ));
            }
        });
    }
}

fn handle_mouse_selection(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut middle_autoscroll: ResMut<MiddleAutoscrollState>,
    mut splitter_drag: ResMut<PanelSplitterDragState>,
    mut mouse_selection: ResMut<MouseSelectionState>,
    panel_query: Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
    text_layout_query: Query<(&PanelText, &TextLayoutInfo)>,
    processed_text_layout_query: Query<
        (&ProcessedPaperText, &TextLayoutInfo, &ComputedNode),
        (Without<PanelText>, Without<PanelPaper>, Without<PanelCaret>, Without<PanelCanvas>),
    >,
    mut state: ResMut<EditorState>,
) {
    if splitter_drag.suppress_next_left_click && !mouse_buttons.pressed(MouseButton::Left) {
        splitter_drag.suppress_next_left_click = false;
    }
    if splitter_drag.suppress_next_left_click || splitter_drag.active.is_some() {
        mouse_selection.active = false;
        return;
    }

    if middle_autoscroll.suppress_next_left_click && !mouse_buttons.pressed(MouseButton::Left) {
        middle_autoscroll.suppress_next_left_click = false;
    }
    if middle_autoscroll.suppress_next_left_click && mouse_buttons.just_pressed(MouseButton::Left) {
        middle_autoscroll.suppress_next_left_click = false;
        mouse_selection.active = false;
        return;
    }
    if middle_autoscroll.is_active() {
        mouse_selection.active = false;
        return;
    }
    if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if mouse_buttons.just_pressed(MouseButton::Left) {
            mouse_selection.active = false;
        }
        return;
    }

    if mouse_selection.active && mouse_buttons.just_released(MouseButton::Left) {
        if !mouse_selection.dragged && !mouse_selection.extend_from_existing {
            state.selection_anchor = None;
        }
        mouse_selection.active = false;
        mouse_selection.dragged = false;
        return;
    }

    let is_start = mouse_buttons.just_pressed(MouseButton::Left);
    let is_drag_update = mouse_selection.active && mouse_buttons.pressed(MouseButton::Left);
    if !is_start && !is_drag_update {
        return;
    }

    let visible_lines = viewport_lines_from_panels(
        &panel_query,
        state.display_mode,
        state.measured_line_step,
        scaled_text_padding_y(&state),
    );
    let plain_panel_size = panel_query
        .iter()
        .find(|(panel, _, _)| panel.kind == PanelKind::Plain)
        .map(|(_, _, computed)| computed.size() * computed.inverse_scale_factor());
    let processed_panel_size = panel_query
        .iter()
        .find(|(panel, _, _)| panel.kind == PanelKind::Processed)
        .map(|(_, _, computed)| computed.size() * computed.inverse_scale_factor());
    state.clamp_horizontal_scrolls(plain_panel_size, processed_panel_size);
    let processed_layout_info =
        processed_panel_size.map(|size| processed_page_layout(size, &state));
    let processed_wrap_columns = processed_layout_info.map_or(64, |layout| layout.wrap_columns);
    let processed_lines_per_page = processed_layout_info.map_or(40, |layout| layout.lines_per_page);
    let processed_spacer_lines = processed_layout_info.map_or(2, |layout| layout.spacer_lines);
    let processed_step_lines = processed_layout_info
        .map_or(processed_page_step_lines(), |layout| layout.page_step_lines)
        .max(1);
    let processed_view_capacity = processed_step_lines
        .saturating_mul(PROCESSED_PAPER_CAPACITY)
        .max(1);
    let plain_lines = visible_plain_lines(&state, visible_lines);
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
        processed_step_lines,
        processed_view_capacity,
    );
    let first_visible_page = processed_view.start_index / processed_step_lines;
    let plain_layout = panel_layout_info(&text_layout_query, PanelKind::Plain);
    let plain_line_height = state.measured_line_step.max(1.0);
    let processed_line_height = scaled_line_height(&state).max(1.0);
    let plain_char_width = scaled_char_width(&state).max(1.0);
    let processed_char_width = scaled_char_width(&state).max(1.0);
    let plain_origin_x = scaled_text_padding_x(&state) - state.plain_horizontal_scroll;
    let plain_origin_y = scaled_text_padding_y(&state);
    let anchor_line_in_page = processed_anchor_line_in_page(&processed_view, processed_step_lines);
    let processed_anchor_offset_px =
        processed_anchor_scroll_offset_px(anchor_line_in_page, processed_line_height);
    let processed_zoom_bias_px = state.processed_zoom_anchor_bias_px;
    let mut hit = None::<(PanelKind, Position)>;

    for (panel, relative_cursor, computed) in panel_query.iter() {
        if !state.panel_visible(panel.kind) {
            continue;
        }
        if !relative_cursor.cursor_over() {
            continue;
        }

        let Some(normalized) = relative_cursor.normalized else {
            continue;
        };

        state.focused_panel = panel.kind;

        if state.document.is_empty() {
            hit = Some((panel.kind, Position::default()));
            break;
        }

        let inverse_scale = computed.inverse_scale_factor();
        let size = computed.size() * inverse_scale;
        let panel_x = (normalized.x + 0.5) * size.x;
        let panel_y = (normalized.y + 0.5) * size.y;
        if panel.kind == PanelKind::Processed {
            let Some(processed_layout) = processed_layout_info else {
                continue;
            };
            if processed_all_lines.is_empty() {
                continue;
            }

            let geometry = processed_layout.geometry;
            let processed_step_px = processed_page_step_px(&geometry, state.zoom);
            let text_left = geometry.text_left - state.processed_horizontal_scroll;
            let text_right = text_left + geometry.text_width;

            let mut clicked_page = None;
            for slot in 0..PROCESSED_PAPER_CAPACITY {
                let page_index = first_visible_page.saturating_add(slot);
                let page_top = processed_page_top_for_slot(
                    &geometry,
                    slot,
                    processed_step_px,
                    processed_anchor_offset_px,
                ) + processed_zoom_bias_px;
                let page_bottom = page_top + geometry.paper_height;

                if panel_y >= page_top && panel_y <= page_bottom {
                    clicked_page = Some((slot, page_index));
                    break;
                }
            }

            let Some((slot, page_index)) = clicked_page else {
                continue;
            };

            let text_top = processed_text_top_for_slot(
                &geometry,
                slot,
                processed_step_px,
                processed_anchor_offset_px,
            ) + processed_zoom_bias_px;
            let local_x = (panel_x - text_left).max(0.0);
            let local_y = (panel_y - text_top).max(0.0);
            let fallback_line_in_page = ((local_y / processed_line_height).floor().max(0.0)
                as usize)
                .min(processed_lines_per_page.saturating_sub(1));
            let fallback_column = if panel_x <= text_left {
                0
            } else {
                ((panel_x.min(text_right) - text_left) / processed_char_width)
                    .round()
                    .max(0.0) as usize
            };

            let (line_in_page, display_column) = processed_text_layout_query
                .iter()
                .find(|(paper_text, _, _)| paper_text.slot == slot)
                .map_or((fallback_line_in_page, fallback_column), |(_, layout, text_computed)| {
                    let inverse_scale = text_computed.inverse_scale_factor();
                    let line_in_page = line_index_from_layout_y(
                        layout,
                        local_y,
                        processed_lines_per_page.max(1),
                        inverse_scale,
                    )
                    .unwrap_or(fallback_line_in_page)
                    .min(processed_lines_per_page.saturating_sub(1));

                    let global_for_line = page_index
                        .saturating_mul(processed_step_lines)
                        .saturating_add(line_in_page)
                        .min(processed_all_lines.len().saturating_sub(1));
                    let display_line =
                        processed_all_lines.get(global_for_line).map_or("", |line| line.text.as_str());
                    let display_column = column_from_layout_x(
                        layout,
                        line_in_page,
                        local_x,
                        display_line,
                        inverse_scale,
                        processed_char_width,
                    )
                    .unwrap_or(fallback_column);
                    (line_in_page, display_column)
                });

            let global_index = page_index
                .saturating_mul(processed_step_lines)
                .saturating_add(line_in_page)
                .min(processed_all_lines.len().saturating_sub(1));
            let Some(global_index) =
                nearest_non_spacer_visual_index(&processed_all_lines, global_index)
            else {
                continue;
            };
            let Some(visual_line) = processed_all_lines.get(global_index) else {
                continue;
            };
            let raw_column = processed_raw_column_from_display(visual_line, display_column);
            let line = visual_line.source_line;
            let max_col = state.document.line_len_chars(line);
            let column = raw_column.min(max_col);
            hit = Some((PanelKind::Processed, Position { line, column }));
            break;
        }

        let local_x = (panel_x - plain_origin_x).max(0.0);
        let local_y = (panel_y - plain_origin_y).max(0.0);
        let panel_line_count = plain_lines.len().max(1);
        let line_offset = plain_layout
            .and_then(|layout| {
                line_index_from_layout_y(layout, local_y, panel_line_count, inverse_scale)
            })
            .unwrap_or_else(|| {
                ((local_y / plain_line_height).floor().max(0.0) as usize)
                    .min(panel_line_count.saturating_sub(1))
            });
        let line = state
            .top_line
            .saturating_add(line_offset)
            .min(state.document.line_count().saturating_sub(1));
        let visible_offset = line.saturating_sub(state.top_line);
        let display_line = plain_lines
            .get(visible_offset)
            .map_or("", |line| line.as_str());
        let raw_column = plain_layout
            .and_then(|layout| {
                column_from_layout_x(
                    layout,
                    visible_offset,
                    local_x,
                    display_line,
                    inverse_scale,
                    plain_char_width,
                )
            })
            .unwrap_or_else(|| (local_x / plain_char_width).round().max(0.0) as usize);

        let max_col = state.document.line_len_chars(line);
        let column = raw_column.min(max_col);
        hit = Some((PanelKind::Plain, Position { line, column }));
        break;
    }

    let Some((_panel, position)) = hit else {
        return;
    };

    if consume_script_link_click(&mut state, &mut mouse_selection, &keys, is_start, position) {
        return;
    }

    if is_start {
        let extend_selection = shift_modifier_pressed(&keys);
        mouse_selection.active = true;
        mouse_selection.extend_from_existing = extend_selection;
        mouse_selection.dragged = false;

        if extend_selection {
            state.set_cursor_with_selection(position, true, true);
        } else {
            state.set_cursor(position, true);
            state.selection_anchor = Some(position);
        }
    } else if mouse_selection.active {
        let previous = state.cursor.position;
        state.set_cursor_with_selection(position, true, true);
        if previous != position {
            mouse_selection.dragged = true;
        }
    }

    apply_cursor_follow_scroll_policy(&mut state, processed_panel_size, visible_lines);
}

fn render_selection_rects(
    selection_rect_query: &mut Query<
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
        ),
    >,
    state: &EditorState,
    plain_lines: &[String],
    plain_layout: Option<&TextLayoutInfo>,
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
        (&ProcessedPaperText, &TextLayoutInfo, &ComputedNode),
        (Without<PanelText>, Without<PanelPaper>, Without<PanelCaret>, Without<PanelCanvas>),
    >,
    processed_geometry: &ProcessedPageGeometry,
    processed_page_step_pixels: f32,
    processed_anchor_offset_px: f32,
    processed_zoom_bias_px: f32,
    processed_char_width: f32,
    processed_line_height: f32,
) {
    let mut plain_rects = Vec::<(f32, f32, f32, f32)>::new();
    let mut processed_rects = Vec::<(f32, f32, f32, f32)>::new();
    if let Some((start, end)) = state.selection_bounds() {
        let visible_first_line = state.top_line;
        let visible_last_line = visible_first_line
            .saturating_add(plain_lines.len().saturating_sub(1));
        let range_start_line = start.line.max(visible_first_line);
        let range_end_line = end.line.min(visible_last_line);

        if range_start_line <= range_end_line {
            for line in range_start_line..=range_end_line {
                if plain_rects.len() >= SELECTION_RECT_CAPACITY {
                    break;
                }

                let visible_offset = line.saturating_sub(visible_first_line);
                let Some(display_line) = plain_lines.get(visible_offset) else {
                    continue;
                };
                let line_len = state.document.line_len_chars(line);
                let line_start = if line == start.line {
                    start.column.min(line_len)
                } else {
                    0
                };
                let line_end = if line == end.line {
                    end.column.min(line_len)
                } else {
                    line_len
                };

                if line_start == line_end {
                    continue;
                }

                let display_len = display_line.chars().count();
                let start_byte = char_to_byte_index(display_line, line_start.min(display_len));
                let end_byte = char_to_byte_index(display_line, line_end.min(display_len));
                let left_x = plain_layout
                    .and_then(|layout| {
                        caret_x_from_layout(
                            layout,
                            visible_offset,
                            display_line,
                            start_byte,
                            plain_inverse_scale,
                            plain_char_width,
                        )
                    })
                    .unwrap_or(line_start as f32 * plain_char_width);
                let right_x = plain_layout
                    .and_then(|layout| {
                        caret_x_from_layout(
                            layout,
                            visible_offset,
                            display_line,
                            end_byte,
                            plain_inverse_scale,
                            plain_char_width,
                        )
                    })
                    .unwrap_or(line_end as f32 * plain_char_width);
                let line_top = plain_layout
                    .and_then(|layout| {
                        line_top_from_layout(layout, visible_offset, plain_inverse_scale)
                    })
                    .unwrap_or(visible_offset as f32 * plain_line_height);

                plain_rects.push((
                    plain_origin_x + left_x.min(right_x),
                    plain_origin_y + line_top,
                    (right_x - left_x).abs().max(1.0),
                    plain_line_height.max(1.0),
                ));
            }
        }

        for (visual_index, visual_line) in processed_view.lines.iter().enumerate() {
            if processed_rects.len() >= SELECTION_RECT_CAPACITY || visual_line.is_spacer {
                continue;
            }

            let source_line = visual_line.source_line;
            if source_line < start.line || source_line > end.line {
                continue;
            }

            let line_len = state.document.line_len_chars(source_line);
            let selected_start_raw = if source_line == start.line {
                start.column.min(line_len)
            } else {
                0
            };
            let selected_end_raw = if source_line == end.line {
                end.column.min(line_len)
            } else {
                line_len
            };
            if selected_end_raw <= selected_start_raw {
                continue;
            }

            let seg_start_raw = visual_line.raw_start_column;
            let seg_end_raw = visual_line.raw_end_column;
            let slice_start_raw = selected_start_raw.max(seg_start_raw);
            let slice_end_raw = selected_end_raw.min(seg_end_raw);
            if slice_end_raw <= slice_start_raw {
                continue;
            }

            let display_start = processed_display_column_from_raw(visual_line, slice_start_raw);
            let display_end = processed_display_column_from_raw(visual_line, slice_end_raw);

            let global_index = processed_view.start_index.saturating_add(visual_index);
            let page_index = global_index / processed_page_step_lines.max(1);
            let line_in_page = global_index % processed_page_step_lines.max(1);
            if line_in_page >= processed_lines_per_page.max(1) {
                continue;
            }
            let slot = page_index.saturating_sub(first_visible_page);
            if slot >= PROCESSED_PAPER_CAPACITY {
                continue;
            }

            let text_left = processed_geometry.text_left - state.processed_horizontal_scroll;
            let text_top = processed_text_top_for_slot(
                processed_geometry,
                slot,
                processed_page_step_pixels,
                processed_anchor_offset_px,
            ) + processed_zoom_bias_px;

            let line_text = visual_line.text.as_str();
            let display_len = line_text.chars().count();
            let start_byte = char_to_byte_index(line_text, display_start.min(display_len));
            let end_byte = char_to_byte_index(line_text, display_end.min(display_len));

            let mut left_x = display_start as f32 * processed_char_width;
            let mut right_x = display_end as f32 * processed_char_width;
            let mut line_top = line_in_page as f32 * processed_line_height;
            let mut line_height = processed_line_height;

            if let Some((_, layout, text_computed)) = processed_text_layout_query
                .iter()
                .find(|(paper_text, _, _)| paper_text.slot == slot)
            {
                let inverse_scale = text_computed.inverse_scale_factor();
                left_x = caret_x_from_layout(
                    layout,
                    line_in_page,
                    line_text,
                    start_byte,
                    inverse_scale,
                    processed_char_width,
                )
                .unwrap_or(left_x);
                right_x = caret_x_from_layout(
                    layout,
                    line_in_page,
                    line_text,
                    end_byte,
                    inverse_scale,
                    processed_char_width,
                )
                .unwrap_or(right_x);
                line_top = line_top_from_layout(layout, line_in_page, inverse_scale).unwrap_or(line_top);
                if let Some((_, top, bottom)) = layout_line_bounds(layout, inverse_scale)
                    .into_iter()
                    .find(|(index, _, _)| *index == line_in_page)
                {
                    line_height = (bottom - top).max(1.0);
                }
            }

            processed_rects.push((
                text_left + left_x.min(right_x),
                text_top + line_top,
                (right_x - left_x).abs().max(1.0),
                line_height.max(1.0),
            ));
        }
    }

    for (selection_rect, mut node, mut color, mut visibility) in selection_rect_query.iter_mut() {
        let rect = match selection_rect.kind {
            PanelKind::Plain => plain_rects.get(selection_rect.index).copied(),
            PanelKind::Processed => processed_rects.get(selection_rect.index).copied(),
        };
        let Some((left, top, width, height)) = rect else {
            *visibility = Visibility::Hidden;
            continue;
        };

        node.left = px(left);
        node.top = px(top);
        node.width = px(width);
        node.height = px(height);
        color.0 = state.selection_bg_color;
        *visibility = Visibility::Visible;
    }
}
