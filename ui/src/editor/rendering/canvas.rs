const COLOR_CANVAS_NODE_BG: Color = Color::srgb(0.96, 0.97, 0.98);
const COLOR_CANVAS_GROUP_BG: Color = Color::srgba(0.80, 0.84, 0.90, 0.28);
const COLOR_CANVAS_NODE_BORDER: Color = Color::srgba(0.08, 0.10, 0.12, 0.22);
const COLOR_CANVAS_NODE_ACTIVE_BORDER: Color = Color::srgb(0.69, 0.28, 0.22);
const COLOR_CANVAS_EDGE: Color = Color::srgba(0.10, 0.12, 0.15, 0.45);
const CANVAS_TEXT_SELECTION_RECT_CAPACITY: usize = 64;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct CanvasRenderedNode {
    index: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct CanvasRenderedNodeText {
    index: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct CanvasRenderedTextSelection {
    node_index: usize,
    rect_index: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct CanvasRenderedTextCaret {
    node_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CanvasTextLayoutRow {
    visual_line: usize,
    source_line: usize,
    min_byte: usize,
    max_byte: usize,
    top: f32,
    bottom: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CanvasTextLayoutSegment {
    visual_line: usize,
    source_line: usize,
    start_byte: usize,
    end_byte: usize,
}

#[derive(Component, Clone, Debug)]
struct CanvasRenderedImage {
    handle: Handle<Image>,
}

#[derive(Component, Clone, Debug)]
struct CanvasRenderedImageError {
    handle: Handle<Image>,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct CanvasRenderedEdge {
    index: usize,
    segment: CanvasEdgeSegment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CanvasEdgeSegment {
    Horizontal,
    Vertical,
}

fn maybe_center_canvas_view_after_layout(
    body_query: &Query<(&PanelBody, &ComputedNode)>,
    state: &mut EditorState,
) {
    if !state.canvas_view_needs_centering {
        return;
    }

    if let Some(processed_panel_size) = body_query
        .iter()
        .find(|(panel, _)| panel.kind == PanelKind::Processed)
        .map(|(_, computed)| computed.size() * computed.inverse_scale_factor())
        .filter(|size| size.x > 1.0 && size.y > 1.0)
    {
        state.center_canvas_view_in_panel(processed_panel_size);
    }
}

fn sync_canvas_board(
    mut commands: Commands,
    canvas_query: Query<(Entity, &PanelCanvas)>,
    existing_node_query: Query<Entity, With<CanvasRenderedNode>>,
    existing_edge_query: Query<Entity, With<CanvasRenderedEdge>>,
    mut node_query: Query<
        (&CanvasRenderedNode, &mut Node, &mut Visibility),
        (
            Without<CanvasRenderedEdge>,
            Without<CanvasRenderedImage>,
            Without<CanvasRenderedImageError>,
            Without<CanvasRenderedTextSelection>,
            Without<CanvasRenderedTextCaret>,
        ),
    >,
    mut edge_query: Query<
        (&CanvasRenderedEdge, &mut Node, &mut Visibility),
        (
            Without<CanvasRenderedNode>,
            Without<CanvasRenderedImage>,
            Without<CanvasRenderedImageError>,
            Without<CanvasRenderedTextSelection>,
            Without<CanvasRenderedTextCaret>,
        ),
    >,
    mut text_query: Query<(&CanvasRenderedNodeText, &mut TextFont, &mut LineHeight)>,
    mut image_query: Query<
        (&CanvasRenderedImage, &mut Visibility),
        (
            Without<CanvasRenderedImageError>,
            Without<CanvasRenderedNode>,
            Without<CanvasRenderedEdge>,
            Without<CanvasRenderedTextSelection>,
            Without<CanvasRenderedTextCaret>,
        ),
    >,
    mut image_error_query: Query<
        (&CanvasRenderedImageError, &mut Visibility),
        (
            Without<CanvasRenderedImage>,
            Without<CanvasRenderedNode>,
            Without<CanvasRenderedEdge>,
            Without<CanvasRenderedTextSelection>,
            Without<CanvasRenderedTextCaret>,
        ),
    >,
    fonts: Res<EditorFonts>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut image_cache: ResMut<EditorImageCache>,
    state: Res<EditorState>,
    mut rendered_version: Local<Option<u64>>,
) {
    if state.document_format != DocumentFormat::Canvas {
        if rendered_version.is_some() {
            for entity in existing_node_query.iter() {
                commands.entity(entity).despawn();
            }
            for entity in existing_edge_query.iter() {
                commands.entity(entity).despawn();
            }
            *rendered_version = None;
        }
        return;
    }

    if *rendered_version != Some(state.canvas_version) {
        for entity in existing_node_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in existing_edge_query.iter() {
            commands.entity(entity).despawn();
        }

        if let (Some(canvas), Some((canvas_entity, _))) = (
            state.canvas_document.as_ref(),
            canvas_query
                .iter()
                .find(|(_, panel_canvas)| panel_canvas.kind == PanelKind::Processed),
        ) {
            commands.entity(canvas_entity).with_children(|parent| {
                for (index, _) in canvas.edges.iter().enumerate() {
                    spawn_canvas_edge(parent, index);
                }
                for (index, node) in canvas.nodes.iter().enumerate() {
                    spawn_canvas_node(
                        parent,
                        index,
                        node,
                        &state,
                        &fonts,
                        &asset_server,
                        &mut images,
                        &mut image_cache,
                    );
                }
            });
        }

        *rendered_version = Some(state.canvas_version);
    }

    let Some(canvas) = state.canvas_document.as_ref() else {
        return;
    };
    let zoom = state.zoom.max(0.1);

    for (rendered_node, mut node, mut visibility) in node_query.iter_mut() {
        let Some(canvas_node) = canvas.nodes.get(rendered_node.index) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let size = canvas_node_size(canvas_node.width, canvas_node.height);
        node.left = px((canvas_node.x - state.canvas_pan.x) * zoom);
        node.top = px((canvas_node.y - state.canvas_pan.y) * zoom);
        node.width = px(size.x * zoom);
        node.height = px(size.y * zoom);
        *visibility = Visibility::Visible;
    }

    for (rendered_edge, mut node, mut visibility) in edge_query.iter_mut() {
        let Some(edge) = canvas.edges.get(rendered_edge.index) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let Some((from_center, to_center)) = canvas_edge_centers(canvas, edge) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let Some((left, top, width, height)) = canvas_edge_segment_rect(
            from_center,
            to_center,
            rendered_edge.segment,
            state.canvas_pan,
            zoom,
        ) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        node.left = px(left);
        node.top = px(top);
        node.width = px(width);
        node.height = px(height);
        *visibility = Visibility::Visible;
    }

    let text_font_size = canvas_text_font_size(zoom);
    let text_line_height = canvas_text_line_height(zoom);
    for (_, mut text_font, mut line_height) in text_query.iter_mut() {
        text_font.font_size = text_font_size;
        *line_height = LineHeight::Px(text_line_height);
    }

    for (canvas_image, mut visibility) in image_query.iter_mut() {
        *visibility = if canvas_image_load_failed(&asset_server, &canvas_image.handle) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }

    for (canvas_image_error, mut visibility) in image_error_query.iter_mut() {
        *visibility = if canvas_image_load_failed(&asset_server, &canvas_image_error.handle) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn sync_canvas_text_overlays(
    text_layout_query: Query<(&CanvasRenderedNodeText, &TextLayoutInfo, &ComputedNode)>,
    mut text_selection_query: Query<
        (
            &CanvasRenderedTextSelection,
            &mut Node,
            &mut BackgroundColor,
            &mut Visibility,
        ),
        (
            Without<CanvasRenderedNode>,
            Without<CanvasRenderedEdge>,
            Without<CanvasRenderedImage>,
            Without<CanvasRenderedImageError>,
            Without<CanvasRenderedTextCaret>,
        ),
    >,
    mut text_caret_query: Query<
        (
            &CanvasRenderedTextCaret,
            &mut Node,
            &mut BackgroundColor,
            &mut Visibility,
            &mut UiTransform,
        ),
        (
            Without<CanvasRenderedNode>,
            Without<CanvasRenderedEdge>,
            Without<CanvasRenderedImage>,
            Without<CanvasRenderedImageError>,
            Without<CanvasRenderedTextSelection>,
        ),
    >,
    state: Res<EditorState>,
) {
    if state.document_format != DocumentFormat::Canvas {
        for (_, mut node, _, mut visibility) in text_selection_query.iter_mut() {
            node.width = px(0.0);
            node.height = px(0.0);
            *visibility = Visibility::Hidden;
        }
        for (_, mut node, _, mut visibility, mut transform) in text_caret_query.iter_mut() {
            node.width = px(0.0);
            node.height = px(0.0);
            transform.scale = Vec2::ONE;
            transform.translation = Val2::ZERO;
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let Some(canvas) = state.canvas_document.as_ref() else {
        return;
    };
    let zoom = state.zoom.max(CANVAS_ZOOM_MIN);
    render_canvas_text_selection_rects(
        &mut text_selection_query,
        &text_layout_query,
        canvas,
        &state,
        zoom,
    );
    render_canvas_text_carets(&mut text_caret_query, &text_layout_query, canvas, &state, zoom);
}

fn spawn_canvas_edge(parent: &mut ChildSpawnerCommands, index: usize) {
    for segment in [CanvasEdgeSegment::Horizontal, CanvasEdgeSegment::Vertical] {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                width: px(0.0),
                height: px(0.0),
                ..default()
            },
            BackgroundColor(COLOR_CANVAS_EDGE),
            ZIndex(10),
            CanvasRenderedEdge { index, segment },
        ));
    }
}

fn spawn_canvas_node(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    node: &basscript_core::CanvasNode,
    state: &EditorState,
    fonts: &EditorFonts,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    image_cache: &mut EditorImageCache,
) {
    let size = canvas_node_size(node.width, node.height);
    let zoom = state.zoom.max(0.1);
    let left = (node.x - state.canvas_pan.x) * zoom;
    let top = (node.y - state.canvas_pan.y) * zoom;
    let node_color = match node.kind {
        CanvasNodeKind::Group { .. } => COLOR_CANVAS_GROUP_BG,
        _ => COLOR_CANVAS_NODE_BG,
    };
    let active_text_node = state.canvas_editing_node_id.as_deref() == Some(node.id.as_str());
    let border_color = if active_text_node {
        COLOR_CANVAS_NODE_ACTIVE_BORDER
    } else {
        COLOR_CANVAS_NODE_BORDER
    };

    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(top),
                width: px(size.x * zoom),
                height: px(size.y * zoom),
                overflow: Overflow::clip(),
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(px(5.0)),
                ..default()
            },
            BackgroundColor(node_color),
            BorderColor::all(border_color),
            ZIndex(20 + index as i32),
            CanvasRenderedNode { index },
        ))
        .with_children(|node_parent| match &node.kind {
            CanvasNodeKind::Text { text } => {
                let text_mode = canvas_text_render_mode(state, active_text_node);
                if text_mode == CanvasTextRenderMode::Rendered {
                    if let Some(target) = canvas_first_image_target(text) {
                        spawn_canvas_image_or_placeholder(
                            node_parent,
                            index,
                            &target,
                            state,
                            fonts,
                            asset_server,
                            images,
                            image_cache,
                            zoom,
                        );
                        return;
                    }
                }

                spawn_canvas_text_preview(node_parent, index, text, text_mode, state, fonts, zoom);
            }
            CanvasNodeKind::File { file } => {
                spawn_canvas_image_or_placeholder(
                    node_parent,
                    index,
                    file,
                    state,
                    fonts,
                    asset_server,
                    images,
                    image_cache,
                    zoom,
                );
            }
            CanvasNodeKind::Link { url } => {
                node_parent.spawn((
                    Text::new(url.clone()),
                    TextFont {
                        font: fonts.markdown_regular.clone(),
                        font_size: canvas_text_font_size(zoom),
                        ..default()
                    },
                    LineHeight::Px(canvas_text_line_height(zoom)),
                    TextColor(COLOR_TEXT_MUTED),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(CANVAS_TEXT_PADDING_X),
                        top: px(CANVAS_TEXT_PADDING_Y),
                        right: px(CANVAS_TEXT_PADDING_X),
                        bottom: px(CANVAS_TEXT_PADDING_Y),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    CanvasRenderedNodeText { index },
                ));
            }
            CanvasNodeKind::Group { label } => {
                node_parent.spawn((
                    Text::new(label.clone().unwrap_or_default()),
                    TextFont {
                        font: fonts.markdown_bold.clone(),
                        font_size: canvas_text_font_size(zoom).max(8.0),
                        ..default()
                    },
                    LineHeight::Px(canvas_text_line_height(zoom)),
                    TextColor(COLOR_TEXT_MUTED),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(CANVAS_TEXT_PADDING_X),
                        top: px(CANVAS_TEXT_PADDING_Y),
                        right: px(CANVAS_TEXT_PADDING_X),
                        bottom: px(CANVAS_TEXT_PADDING_Y),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    CanvasRenderedNodeText { index },
                ));
            }
            CanvasNodeKind::Unknown { node_type } => {
                node_parent.spawn((
                    Text::new(format!("Unsupported canvas node: {node_type}")),
                    TextFont {
                        font: fonts.markdown_regular.clone(),
                        font_size: canvas_text_font_size(zoom),
                        ..default()
                    },
                    LineHeight::Px(canvas_text_line_height(zoom)),
                    TextColor(COLOR_TEXT_MUTED),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(CANVAS_TEXT_PADDING_X),
                        top: px(CANVAS_TEXT_PADDING_Y),
                        right: px(CANVAS_TEXT_PADDING_X),
                        bottom: px(CANVAS_TEXT_PADDING_Y),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    CanvasRenderedNodeText { index },
                ));
            }
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CanvasTextRenderMode {
    Rendered,
    Plain,
    PlainInteractive,
}

fn spawn_canvas_text_preview(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    text: &str,
    mode: CanvasTextRenderMode,
    state: &EditorState,
    fonts: &EditorFonts,
    zoom: f32,
) {
    let preview = canvas_text_preview(text, mode, state);
    if mode == CanvasTextRenderMode::PlainInteractive {
        spawn_canvas_text_selection_rects(parent, index);
        spawn_canvas_text_caret(parent, index);
    }

    let text_font = match mode {
        CanvasTextRenderMode::Rendered => fonts.markdown_regular.clone(),
        CanvasTextRenderMode::Plain | CanvasTextRenderMode::PlainInteractive => fonts.regular.clone(),
    };
    parent
        .spawn((
            Text::new(preview.text),
            TextFont {
                font: text_font,
                font_size: canvas_text_font_size(zoom),
                ..default()
            },
            LineHeight::Px(canvas_text_line_height(zoom)),
            TextColor(COLOR_TEXT_MAIN),
            Node {
                position_type: PositionType::Absolute,
                left: px(CANVAS_TEXT_PADDING_X),
                top: px(CANVAS_TEXT_PADDING_Y),
                right: px(CANVAS_TEXT_PADDING_X),
                bottom: px(CANVAS_TEXT_PADDING_Y),
                overflow: Overflow::clip(),
                ..default()
            },
            ZIndex(1),
            CanvasRenderedNodeText { index },
        ));
}

#[derive(Default)]
struct CanvasTextPreview {
    text: String,
}

fn spawn_canvas_text_selection_rects(parent: &mut ChildSpawnerCommands, node_index: usize) {
    for rect_index in 0..CANVAS_TEXT_SELECTION_RECT_CAPACITY {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                width: px(0.0),
                height: px(0.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Visibility::Hidden,
            ZIndex(0),
            CanvasRenderedTextSelection {
                node_index,
                rect_index,
            },
        ));
    }
}

fn spawn_canvas_text_caret(parent: &mut ChildSpawnerCommands, node_index: usize) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            top: px(0.0),
            width: px(CARET_WIDTH),
            height: px(LINE_HEIGHT),
            ..default()
        },
        UiTransform::default(),
        BackgroundColor(Color::srgba(0.12, 0.12, 0.13, 0.35)),
        Visibility::Hidden,
        ZIndex(2),
        CanvasRenderedTextCaret { node_index },
    ));
}

fn render_canvas_text_selection_rects(
    selection_query: &mut Query<
        (
            &CanvasRenderedTextSelection,
            &mut Node,
            &mut BackgroundColor,
            &mut Visibility,
        ),
        (
            Without<CanvasRenderedNode>,
            Without<CanvasRenderedEdge>,
            Without<CanvasRenderedImage>,
            Without<CanvasRenderedImageError>,
            Without<CanvasRenderedTextCaret>,
        ),
    >,
    text_layout_query: &Query<(&CanvasRenderedNodeText, &TextLayoutInfo, &ComputedNode)>,
    canvas: &CanvasDocument,
    state: &EditorState,
    zoom: f32,
) {
    let active_node_index = canvas_active_text_node_index(canvas, state);
    let rects = active_node_index
        .map(|node_index| {
            canvas_text_selection_rects(canvas, node_index, state, text_layout_query, zoom)
        })
        .unwrap_or_default();

    for (selection, mut node, mut color, mut visibility) in selection_query.iter_mut() {
        if Some(selection.node_index) != active_node_index || selection.rect_index >= rects.len() {
            *visibility = Visibility::Hidden;
            continue;
        }

        let (left, top, width, height) = rects[selection.rect_index];
        node.left = px(left);
        node.top = px(top);
        node.width = px(width);
        node.height = px(height);
        color.0 = state.selection_bg_color;
        *visibility = Visibility::Visible;
    }
}

fn render_canvas_text_carets(
    caret_query: &mut Query<
        (
            &CanvasRenderedTextCaret,
            &mut Node,
            &mut BackgroundColor,
            &mut Visibility,
            &mut UiTransform,
        ),
        (
            Without<CanvasRenderedNode>,
            Without<CanvasRenderedEdge>,
            Without<CanvasRenderedImage>,
            Without<CanvasRenderedImageError>,
            Without<CanvasRenderedTextSelection>,
        ),
    >,
    text_layout_query: &Query<(&CanvasRenderedNodeText, &TextLayoutInfo, &ComputedNode)>,
    canvas: &CanvasDocument,
    state: &EditorState,
    zoom: f32,
) {
    let active_node_index = canvas_active_text_node_index(canvas, state);
    let caret_rect = active_node_index
        .filter(|_| state.caret_visible)
        .and_then(|node_index| canvas_text_caret_rect(canvas, node_index, state, text_layout_query, zoom));

    for (caret, mut node, mut color, mut visibility, mut transform) in caret_query.iter_mut() {
        if Some(caret.node_index) != active_node_index || caret_rect.is_none() {
            *visibility = Visibility::Hidden;
            continue;
        }

        let (left, top, width, height) = caret_rect.unwrap();
        node.left = px(left);
        node.top = px(top);
        node.width = px(width);
        node.height = px(height);
        color.0 = Color::srgba(0.12, 0.12, 0.13, 0.35);
        transform.scale = Vec2::ONE;
        transform.translation = Val2::ZERO;
        *visibility = Visibility::Visible;
    }
}

fn canvas_active_text_node_index(canvas: &CanvasDocument, state: &EditorState) -> Option<usize> {
    let node_id = state.canvas_editing_node_id.as_deref()?;
    canvas.nodes.iter().position(|node| {
        node.id == node_id && matches!(node.kind, CanvasNodeKind::Text { .. })
    })
}

fn canvas_text_selection_rects(
    canvas: &CanvasDocument,
    node_index: usize,
    state: &EditorState,
    text_layout_query: &Query<(&CanvasRenderedNodeText, &TextLayoutInfo, &ComputedNode)>,
    zoom: f32,
) -> Vec<(f32, f32, f32, f32)> {
    let Some(node) = canvas.nodes.get(node_index) else {
        return Vec::new();
    };
    let CanvasNodeKind::Text { text } = &node.kind else {
        return Vec::new();
    };
    let document = Document::from_text(text);
    let Some((start, end)) = state.canvas_text_selection_bounds(&document) else {
        return Vec::new();
    };

    let layout = canvas_text_layout_for_node(text_layout_query, node_index);
    let fallback_char_width = canvas_text_char_width(zoom);
    let fallback_line_height = canvas_text_line_height(zoom);
    let mut rects = Vec::new();
    for line in start.line..=end.line {
        if rects.len() >= CANVAS_TEXT_SELECTION_RECT_CAPACITY {
            break;
        }

        let line_len = document.line_len_chars(line);
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

        let line_text = document.line(line).unwrap_or_default();
        let display_len = line_text.chars().count();
        let start_byte = char_to_byte_index(line_text, line_start.min(display_len));
        let end_byte = char_to_byte_index(line_text, line_end.min(display_len));
        if let Some((layout, inverse_scale)) = layout {
            let segments =
                canvas_text_layout_segments_for_line(layout, &document, line, inverse_scale);
            if !segments.is_empty() {
                for segment in segments {
                    if rects.len() >= CANVAS_TEXT_SELECTION_RECT_CAPACITY {
                        break;
                    }

                    let segment_start = start_byte.max(segment.start_byte);
                    let segment_end = end_byte.min(segment.end_byte);
                    if segment_end <= segment_start {
                        continue;
                    }

                    let start_layout_byte =
                        canvas_text_segment_layout_byte(segment, segment_start);
                    let end_layout_byte = canvas_text_segment_layout_byte(segment, segment_end);
                    let left_x = canvas_caret_x_from_layout(
                        layout,
                        segment,
                        line_text,
                        start_layout_byte,
                        inverse_scale,
                        fallback_char_width,
                    )
                    .unwrap_or(start_layout_byte as f32 * fallback_char_width);
                    let right_x = canvas_caret_x_from_layout(
                        layout,
                        segment,
                        line_text,
                        end_layout_byte,
                        inverse_scale,
                        fallback_char_width,
                    )
                    .unwrap_or(end_layout_byte as f32 * fallback_char_width);
                    let line_top = canvas_text_visual_line_top(
                        layout,
                        segment.visual_line,
                        inverse_scale,
                    )
                        .unwrap_or(segment.visual_line as f32 * fallback_line_height);

                    rects.push((
                        CANVAS_TEXT_PADDING_X + left_x.min(right_x),
                        CANVAS_TEXT_PADDING_Y
                            + (line_top + caret_vertical_offset(fallback_line_height)).max(0.0),
                        (right_x - left_x).abs().max(1.0),
                        fallback_line_height.max(1.0),
                    ));
                }
                continue;
            }
        }

        let left_x = line_start.min(display_len) as f32 * fallback_char_width;
        let right_x = line_end.min(display_len) as f32 * fallback_char_width;
        let line_top = line as f32 * fallback_line_height;

        rects.push((
            CANVAS_TEXT_PADDING_X + left_x.min(right_x),
            CANVAS_TEXT_PADDING_Y
                + (line_top + caret_vertical_offset(fallback_line_height)).max(0.0),
            (right_x - left_x).abs().max(1.0),
            fallback_line_height.max(1.0),
        ));
    }

    rects
}

fn canvas_text_caret_rect(
    canvas: &CanvasDocument,
    node_index: usize,
    state: &EditorState,
    text_layout_query: &Query<(&CanvasRenderedNodeText, &TextLayoutInfo, &ComputedNode)>,
    zoom: f32,
) -> Option<(f32, f32, f32, f32)> {
    let node = canvas.nodes.get(node_index)?;
    let CanvasNodeKind::Text { text } = &node.kind else {
        return None;
    };
    let document = Document::from_text(text);
    let cursor = document.clamp_position(state.canvas_text_cursor.position);
    let line_text = document.line(cursor.line).unwrap_or_default();
    let display_len = line_text.chars().count();
    let byte_index = char_to_byte_index(line_text, cursor.column.min(display_len));
    let fallback_char_width = canvas_text_char_width(zoom);
    let fallback_line_height = canvas_text_line_height(zoom);
    let mut caret_x = cursor.column as f32 * fallback_char_width;
    let mut caret_top = cursor.line as f32 * fallback_line_height;

    if let Some((layout, inverse_scale)) = canvas_text_layout_for_node(text_layout_query, node_index)
    {
        if let Some(segment) =
            canvas_text_layout_segment_for_position(
                layout,
                &document,
                cursor.line,
                byte_index,
                inverse_scale,
            )
        {
            let layout_byte = canvas_text_segment_layout_byte(segment, byte_index);
            caret_x = canvas_caret_x_from_layout(
                layout,
                segment,
                line_text,
                layout_byte,
                inverse_scale,
                fallback_char_width,
            )
            .unwrap_or(caret_x);
            caret_top = canvas_text_visual_line_top(layout, segment.visual_line, inverse_scale)
                .unwrap_or(caret_top);
        }
    }

    Some((
        CANVAS_TEXT_PADDING_X + (caret_x + caret_x_offset_for_state(state)).max(0.0),
        CANVAS_TEXT_PADDING_Y + (caret_top + caret_vertical_offset(fallback_line_height)).max(0.0),
        caret_width_for_state(state, fallback_char_width),
        fallback_line_height.max(1.0),
    ))
}

fn canvas_text_layout_for_node<'a>(
    text_layout_query: &'a Query<(&CanvasRenderedNodeText, &TextLayoutInfo, &ComputedNode)>,
    node_index: usize,
) -> Option<(&'a TextLayoutInfo, f32)> {
    text_layout_query
        .iter()
        .find(|(node_text, _, _)| node_text.index == node_index)
        .map(|(_, layout, computed)| (layout, computed.inverse_scale_factor()))
}

fn canvas_text_layout_visual_line_count(layout: &TextLayoutInfo, inverse_scale: f32) -> usize {
    canvas_text_visual_row_bounds(layout, inverse_scale)
        .len()
        .max(1)
}

fn canvas_text_visual_line_from_y(
    layout: &TextLayoutInfo,
    y: f32,
    inverse_scale: f32,
) -> Option<usize> {
    let rows = canvas_text_visual_row_bounds(layout, inverse_scale);
    rows.iter()
        .min_by(|(_, top_a, bottom_a), (_, top_b, bottom_b)| {
            let center_a = (*top_a + *bottom_a) * 0.5;
            let center_b = (*top_b + *bottom_b) * 0.5;
            (center_a - y)
                .abs()
                .partial_cmp(&(center_b - y).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(visual_line, _, _)| *visual_line)
}

fn canvas_text_visual_line_top(
    layout: &TextLayoutInfo,
    visual_line: usize,
    inverse_scale: f32,
) -> Option<f32> {
    canvas_text_layout_rows(layout, inverse_scale)
        .into_iter()
        .find(|row| row.visual_line == visual_line)
        .map(|row| row.top)
        .or_else(|| {
            canvas_text_visual_row_bounds(layout, inverse_scale)
                .into_iter()
                .find(|(row, _, _)| *row == visual_line)
                .map(|(_, top, _)| top)
        })
}

fn canvas_text_position_from_layout(
    document: &Document,
    layout: &TextLayoutInfo,
    visual_line: usize,
    x: f32,
    inverse_scale: f32,
    fallback_char_width: f32,
) -> Option<Position> {
    for line in 0..document.line_count() {
        let line_text = document.line(line).unwrap_or_default();
        let Some(segment) = canvas_text_layout_segments_for_line(layout, document, line, inverse_scale)
            .into_iter()
            .find(|segment| segment.visual_line == visual_line)
        else {
            continue;
        };

        let layout_byte = canvas_byte_from_layout_x(
            layout,
            segment,
            x,
            line_text,
            inverse_scale,
            fallback_char_width,
        )
        .unwrap_or_else(|| (x / fallback_char_width).round().max(0.0) as usize);
        let document_byte = canvas_text_segment_document_byte(segment, layout_byte);

        return Some(Position {
            line,
            column: byte_to_char_index(line_text, document_byte).min(document.line_len_chars(line)),
        });
    }

    None
}

fn canvas_text_layout_segment_for_position(
    layout: &TextLayoutInfo,
    document: &Document,
    line: usize,
    byte_index: usize,
    inverse_scale: f32,
) -> Option<CanvasTextLayoutSegment> {
    let segments = canvas_text_layout_segments_for_line(layout, document, line, inverse_scale);
    segments
        .iter()
        .rev()
        .find(|segment| byte_index >= segment.start_byte)
        .copied()
        .or_else(|| segments.first().copied())
}

fn canvas_text_layout_segments_for_line(
    layout: &TextLayoutInfo,
    document: &Document,
    target_line: usize,
    inverse_scale: f32,
) -> Vec<CanvasTextLayoutSegment> {
    let Some(line_text) = document.line(target_line) else {
        return Vec::new();
    };
    let line_len = line_text.len();
    if line_len == 0 {
        return Vec::new();
    }

    let mut rows = canvas_text_layout_rows(layout, inverse_scale)
        .into_iter()
        .filter(|row| row.source_line == target_line)
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.visual_line);

    let row_starts = rows
        .iter()
        .map(|row| row.min_byte.min(line_len))
        .collect::<Vec<_>>();

    rows.into_iter()
        .enumerate()
        .filter_map(|(index, row)| {
            let start_byte = if index == 0 {
                0
            } else {
                row_starts[index].min(line_len)
            };
            let end_byte = row_starts
                .get(index.saturating_add(1))
                .copied()
                .unwrap_or(line_len)
                .max(start_byte)
                .min(line_len);
            (end_byte > start_byte).then_some(CanvasTextLayoutSegment {
                visual_line: row.visual_line,
                source_line: target_line,
                start_byte,
                end_byte,
            })
        })
        .collect()
}

fn canvas_text_layout_rows(layout: &TextLayoutInfo, inverse_scale: f32) -> Vec<CanvasTextLayoutRow> {
    let row_bounds = canvas_text_visual_row_bounds(layout, inverse_scale);
    let mut rows = BTreeMap::<usize, (usize, usize, usize, f32, f32)>::new();
    for glyph in &layout.glyphs {
        let Some(visual_line) = canvas_glyph_visual_line(&row_bounds, glyph, inverse_scale) else {
            continue;
        };
        let start = glyph.byte_index;
        let end = glyph.byte_index.saturating_add(glyph.byte_length);
        let top = glyph.position.y * inverse_scale;
        let bottom = (glyph.position.y + glyph.size.y) * inverse_scale;
        let entry = rows
            .entry(visual_line)
            .or_insert((glyph.line_index, start, end, top, bottom));
        entry.0 = entry.0.min(glyph.line_index);
        entry.1 = entry.1.min(start);
        entry.2 = entry.2.max(end);
        entry.3 = entry.3.min(top);
        entry.4 = entry.4.max(bottom);
    }

    rows.into_iter()
        .map(
            |(visual_line, (source_line, min_byte, max_byte, top, bottom))| {
                CanvasTextLayoutRow {
            visual_line,
                    source_line,
            min_byte,
            max_byte,
                    top,
                    bottom,
                }
            },
        )
        .collect()
}

fn canvas_text_segment_layout_byte(segment: CanvasTextLayoutSegment, document_byte: usize) -> usize {
    document_byte.clamp(segment.start_byte, segment.end_byte)
}

fn canvas_text_segment_document_byte(segment: CanvasTextLayoutSegment, layout_byte: usize) -> usize {
    layout_byte.clamp(segment.start_byte, segment.end_byte)
}

fn canvas_byte_from_layout_x(
    layout: &TextLayoutInfo,
    segment: CanvasTextLayoutSegment,
    x: f32,
    line_text: &str,
    inverse_scale: f32,
    fallback_char_width: f32,
) -> Option<usize> {
    let boundaries = canvas_line_boundaries(
        layout,
        segment,
        line_text,
        inverse_scale,
        fallback_char_width,
    );
    let (best_byte, _) = boundaries.iter().min_by(|(_, ax), (_, bx)| {
        (*ax - x)
            .abs()
            .partial_cmp(&(*bx - x).abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    })?;
    Some(*best_byte)
}

fn canvas_caret_x_from_layout(
    layout: &TextLayoutInfo,
    segment: CanvasTextLayoutSegment,
    line_text: &str,
    byte_index: usize,
    inverse_scale: f32,
    fallback_char_width: f32,
) -> Option<f32> {
    let boundaries = canvas_line_boundaries(
        layout,
        segment,
        line_text,
        inverse_scale,
        fallback_char_width,
    );
    boundaries
        .iter()
        .find(|(byte, _)| *byte >= byte_index)
        .map(|(_, x)| *x)
        .or_else(|| boundaries.last().map(|(_, x)| *x))
}

fn canvas_line_boundaries(
    layout: &TextLayoutInfo,
    segment: CanvasTextLayoutSegment,
    line_text: &str,
    inverse_scale: f32,
    fallback_char_width: f32,
) -> Vec<(usize, f32)> {
    let line_len = line_text.len();
    let row_bounds = canvas_text_visual_row_bounds(layout, inverse_scale);
    let mut glyphs = layout
        .glyphs
        .iter()
        .filter(|glyph| {
            glyph.line_index == segment.source_line
                && canvas_glyph_visual_line(&row_bounds, glyph, inverse_scale)
                    == Some(segment.visual_line)
        })
        .collect::<Vec<_>>();

    if glyphs.is_empty() {
        let mut boundaries = Vec::with_capacity(line_len.saturating_add(1));
        for local_byte in 0..=line_len {
            boundaries.push((local_byte, local_byte as f32 * fallback_char_width));
        }
        return boundaries;
    }

    glyphs.sort_by_key(|glyph| (glyph.byte_index, glyph.byte_length));
    let mut step_candidates = glyphs
        .windows(2)
        .filter_map(|window| {
            let left = window[0];
            let right = window[1];
            let byte_gap = right.byte_index.saturating_sub(left.byte_index);
            if byte_gap == 0 {
                return None;
            }
            let step = (right.position.x - left.position.x) * inverse_scale / byte_gap as f32;
            (step.is_finite() && step.abs() > 0.1).then_some(step)
        })
        .collect::<Vec<_>>();

    step_candidates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let byte_step = step_candidates
        .get(step_candidates.len().saturating_sub(1) / 2)
        .copied()
        .unwrap_or(fallback_char_width);

    let mut anchors = BTreeMap::<usize, Vec<f32>>::new();

    for glyph in glyphs {
        let start = glyph.byte_index.min(line_len);
        let end = glyph
            .byte_index
            .saturating_add(glyph.byte_length)
            .min(line_len);
        let span_bytes = end.saturating_sub(start).max(1);
        let half_width = byte_step * span_bytes as f32 * 0.5;
        let center_x = glyph.position.x * inverse_scale;
        let left = center_x - half_width;
        let right = center_x + half_width;

        anchors.entry(start).or_default().push(left);
        anchors.entry(end).or_default().push(right);
    }

    let mut known = anchors
        .into_iter()
        .map(|(byte_index, xs)| {
            let sum = xs.iter().copied().sum::<f32>();
            (byte_index, sum / xs.len() as f32)
        })
        .collect::<Vec<_>>();

    known.sort_by_key(|(byte_index, _)| *byte_index);

    let first = known[0];
    let last = known[known.len().saturating_sub(1)];
    let mut boundaries = Vec::with_capacity(line_len.saturating_add(1));
    let mut segment = 0usize;

    for local_byte in 0..=line_len {
        let byte_index = local_byte;
        while segment + 1 < known.len() && known[segment + 1].0 <= byte_index {
            segment += 1;
        }

        let x = if byte_index <= first.0 {
            first.1 - first.0.saturating_sub(byte_index) as f32 * byte_step
        } else if byte_index >= last.0 {
            last.1 + byte_index.saturating_sub(last.0) as f32 * byte_step
        } else {
            let (left_byte, left_x) = known[segment];
            let (right_byte, right_x) = known[segment + 1];
            let gap = right_byte.saturating_sub(left_byte).max(1);
            let t = byte_index.saturating_sub(left_byte) as f32 / gap as f32;
            left_x + (right_x - left_x) * t
        };

        boundaries.push((byte_index, x));
    }

    boundaries
}

fn canvas_text_visual_row_bounds(
    layout: &TextLayoutInfo,
    inverse_scale: f32,
) -> Vec<(usize, f32, f32)> {
    let mut bounds = layout
        .run_geometry
        .iter()
        .map(|run| (run.bounds.min.y, run.bounds.max.y))
        .filter(|(top, bottom)| top.is_finite() && bottom.is_finite())
        .collect::<Vec<_>>();

    if bounds.is_empty() {
        bounds = layout
            .glyphs
            .iter()
            .map(|glyph| {
                let top = glyph.position.y * inverse_scale;
                let bottom = (glyph.position.y + glyph.size.y) * inverse_scale;
                (top, bottom)
            })
            .filter(|(top, bottom)| top.is_finite() && bottom.is_finite())
            .collect();
    }

    bounds.sort_by(|(top_a, _), (top_b, _)| {
        top_a
            .partial_cmp(top_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut merged = Vec::<(f32, f32)>::new();
    for (top, bottom) in bounds {
        if let Some((last_top, last_bottom)) = merged.last_mut()
            && (top - *last_top).abs() <= 0.5
        {
            *last_top = (*last_top).min(top);
            *last_bottom = (*last_bottom).max(bottom);
            continue;
        }

        merged.push((top, bottom.max(top + 1.0)));
    }

    merged
        .into_iter()
        .enumerate()
        .map(|(index, (top, bottom))| (index, top, bottom))
        .collect()
}

fn canvas_glyph_visual_line(
    row_bounds: &[(usize, f32, f32)],
    glyph: &bevy::text::PositionedGlyph,
    inverse_scale: f32,
) -> Option<usize> {
    let glyph_y = glyph.position.y * inverse_scale;
    row_bounds
        .iter()
        .min_by(|(_, top_a, bottom_a), (_, top_b, bottom_b)| {
            let center_a = (*top_a + *bottom_a) * 0.5;
            let center_b = (*top_b + *bottom_b) * 0.5;
            (center_a - glyph_y)
                .abs()
                .partial_cmp(&(center_b - glyph_y).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(visual_line, _, _)| *visual_line)
}

fn spawn_canvas_image_or_placeholder(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    target: &str,
    state: &EditorState,
    fonts: &EditorFonts,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    image_cache: &mut EditorImageCache,
    zoom: f32,
) {
    if is_remote_image_target(target) && !remote_image_target_seems_loadable(target) {
        spawn_canvas_image_error(
            parent,
            index,
            "Image not loaded: expected a direct image URL",
            target,
            fonts,
            zoom,
        );
        return;
    }

    let lookup = processed_image_lookup(image_cache, state, target, asset_server, images);
    match lookup {
        ProcessedImageLookup::Loaded { handle, .. } => {
            let image_handle = handle.clone();
            parent.spawn((
                ImageNode::new(image_handle.clone()),
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
                CanvasRenderedImage {
                    handle: image_handle.clone(),
                },
            ));

            if is_remote_image_target(target) {
                parent.spawn((
                    Text::new(canvas_image_error_text("Image failed to load", target)),
                    TextFont {
                        font: fonts.markdown_regular.clone(),
                        font_size: canvas_text_font_size(zoom),
                        ..default()
                    },
                    LineHeight::Px(canvas_text_line_height(zoom)),
                    TextColor(COLOR_TEXT_MUTED),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(CANVAS_TEXT_PADDING_X),
                        top: px(CANVAS_TEXT_PADDING_Y),
                        right: px(CANVAS_TEXT_PADDING_X),
                        bottom: px(CANVAS_TEXT_PADDING_Y),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    Visibility::Hidden,
                    CanvasRenderedNodeText { index },
                    CanvasRenderedImageError {
                        handle: image_handle,
                    },
                ));
            }
        }
        ProcessedImageLookup::Failed => {
            spawn_canvas_image_error(parent, index, "Image failed to load", target, fonts, zoom);
        }
    }
}

fn spawn_canvas_image_error(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    message: &str,
    target: &str,
    fonts: &EditorFonts,
    zoom: f32,
) {
    parent.spawn((
        Text::new(canvas_image_error_text(message, target)),
        TextFont {
            font: fonts.markdown_regular.clone(),
            font_size: canvas_text_font_size(zoom),
            ..default()
        },
        LineHeight::Px(canvas_text_line_height(zoom)),
        TextColor(COLOR_TEXT_MUTED),
        Node {
            position_type: PositionType::Absolute,
            left: px(CANVAS_TEXT_PADDING_X),
            top: px(CANVAS_TEXT_PADDING_Y),
            right: px(CANVAS_TEXT_PADDING_X),
            bottom: px(CANVAS_TEXT_PADDING_Y),
            overflow: Overflow::clip(),
            ..default()
        },
        CanvasRenderedNodeText { index },
    ));
}

fn canvas_image_load_failed(asset_server: &AssetServer, handle: &Handle<Image>) -> bool {
    matches!(asset_server.get_load_state(handle), Some(LoadState::Failed(_)))
}

fn canvas_image_error_text(message: &str, target: &str) -> String {
    format!("{message}\n{}", canvas_file_placeholder(target))
}

fn canvas_first_image_target(text: &str) -> Option<String> {
    canvas_first_markdown_image_embed(text)
        .map(|embed| embed.target)
        .or_else(|| canvas_first_html_image_src(text))
        .or_else(|| canvas_first_obsidian_image_target(text))
        .or_else(|| canvas_first_remote_image_url(text))
        .and_then(|target| canvas_clean_image_target(&target))
}

fn remote_image_target_seems_loadable(target: &str) -> bool {
    let trimmed = target.trim();
    if !is_remote_image_target(trimmed) {
        return true;
    }
    let without_fragment = trimmed.split('#').next().unwrap_or(trimmed);
    let without_query = without_fragment.split('?').next().unwrap_or(without_fragment);
    let lower = without_query.to_ascii_lowercase();
    matches!(
        Path::new(&lower).extension().and_then(|ext| ext.to_str()),
        Some("png" | "jpg" | "jpeg" | "webp" | "bmp")
    )
}

fn canvas_first_markdown_image_embed(text: &str) -> Option<ImageEmbed> {
    parse_document_with_format(&Document::from_text(text), DocumentFormat::Markdown)
        .into_iter()
        .flat_map(|line| line.image_embeds)
        .next()
}

fn canvas_first_html_image_src(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let mut search_start = 0usize;

    while let Some(relative_start) = lower.get(search_start..)?.find("<img") {
        let tag_start = search_start + relative_start;
        let tag_end = lower
            .get(tag_start..)?
            .find('>')
            .map_or(text.len(), |offset| tag_start + offset);
        let tag = text.get(tag_start..tag_end)?;
        if let Some(src) = html_attr_value(tag, "src") {
            return Some(src);
        }
        search_start = tag_end.saturating_add(1);
    }

    None
}

fn canvas_first_obsidian_image_target(text: &str) -> Option<String> {
    let start = text.find("![[")? + 3;
    let rest = text.get(start..)?;
    let end = rest.find("]]").or_else(|| rest.find(']')).unwrap_or(rest.len());
    let target = rest.get(..end)?;
    Some(target.split('|').next().unwrap_or(target).trim().to_owned())
}

fn canvas_first_remote_image_url(text: &str) -> Option<String> {
    let start = text.find("https://").or_else(|| text.find("http://"))?;
    let rest = text.get(start..)?;
    let end = rest
        .char_indices()
        .find_map(|(index, ch)| {
            (ch.is_whitespace() || matches!(ch, '"' | '\'' | '<' | '>' | ')' | ']' | '|'))
                .then_some(index)
        })
        .unwrap_or(rest.len());
    Some(rest.get(..end)?.to_owned())
}

fn canvas_clean_image_target(target: &str) -> Option<String> {
    let mut trimmed = target.trim();
    if let Some(markdown_target_start) = trimmed.find("](") {
        let target_start = markdown_target_start + 2;
        let target_end = trimmed
            .get(target_start..)
            .and_then(|rest| rest.find(')').map(|offset| target_start + offset))
            .unwrap_or(trimmed.len());
        trimmed = trimmed.get(target_start..target_end)?.trim();
    }
    if trimmed.starts_with('<') && trimmed.ends_with('>') && trimmed.len() > 2 {
        trimmed = &trimmed[1..trimmed.len() - 1];
    }
    if let Some((left, _)) = trimmed.split_once('|') {
        if !left.trim().is_empty() {
            trimmed = left.trim();
        }
    }
    let cleaned = trimmed
        .trim_matches(|ch: char| matches!(ch, '"' | '\'' | '<' | '>' | ')' | ']' | '['))
        .trim();
    (!cleaned.is_empty()).then(|| cleaned.to_owned())
}

fn html_attr_value(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let mut search_start = 0usize;

    while let Some(relative_start) = lower.get(search_start..)?.find(attr) {
        let attr_start = search_start + relative_start;
        let after_attr = attr_start + attr.len();
        let bytes = lower.as_bytes();
        let before_ok = attr_start == 0
            || !bytes
                .get(attr_start.saturating_sub(1))
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'-');
        let after_ok = bytes
            .get(after_attr)
            .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b'=');
        if !before_ok || !after_ok {
            search_start = after_attr;
            continue;
        }

        let mut cursor = after_attr;
        cursor = skip_ascii_whitespace(tag, cursor);
        if tag.as_bytes().get(cursor) != Some(&b'=') {
            search_start = after_attr;
            continue;
        }
        cursor = skip_ascii_whitespace(tag, cursor + 1);

        let bytes = tag.as_bytes();
        let value = match bytes.get(cursor) {
            Some(b'"') | Some(b'\'') => {
                let quote = bytes[cursor];
                let start = cursor + 1;
                let end = bytes[start..]
                    .iter()
                    .position(|byte| *byte == quote)
                    .map(|offset| start + offset)?;
                tag.get(start..end)?.to_owned()
            }
            Some(_) => {
                let start = cursor;
                let end = bytes[start..]
                    .iter()
                    .position(|byte| byte.is_ascii_whitespace() || *byte == b'>')
                    .map_or(tag.len(), |offset| start + offset);
                tag.get(start..end)?.to_owned()
            }
            None => return None,
        };

        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_owned());
        }

        search_start = after_attr;
    }

    None
}

fn skip_ascii_whitespace(input: &str, mut index: usize) -> usize {
    while input
        .as_bytes()
        .get(index)
        .is_some_and(u8::is_ascii_whitespace)
    {
        index += 1;
    }
    index
}

fn canvas_text_render_mode(state: &EditorState, active_text_node: bool) -> CanvasTextRenderMode {
    if state.display_mode == DisplayMode::Plain {
        return if active_text_node {
            CanvasTextRenderMode::PlainInteractive
        } else {
            CanvasTextRenderMode::Plain
        };
    }

    if state.display_mode == DisplayMode::ProcessedRawCurrentLine && active_text_node {
        return CanvasTextRenderMode::PlainInteractive;
    }

    if active_text_node && canvas_text_interaction_active(state) {
        CanvasTextRenderMode::PlainInteractive
    } else {
        CanvasTextRenderMode::Rendered
    }
}

fn canvas_text_interaction_active(state: &EditorState) -> bool {
    state.canvas_editing_node_id.is_some()
        && (!state.vim_enabled
            || matches!(
                state.vim_mode,
                VimMode::Normal | VimMode::Insert | VimMode::VisualChar | VimMode::VisualLine
            ))
}

fn canvas_text_preview(
    text: &str,
    mode: CanvasTextRenderMode,
    _state: &EditorState,
) -> CanvasTextPreview {
    let normalized = text.replace("\r\n", "\n");
    match mode {
        CanvasTextRenderMode::Rendered => CanvasTextPreview {
            text: normalized
                .lines()
                .map(|line| line.strip_prefix("# ").unwrap_or(line))
                .collect::<Vec<_>>()
                .join("\n"),
        },
        CanvasTextRenderMode::Plain => CanvasTextPreview {
            text: normalized,
        },
        CanvasTextRenderMode::PlainInteractive => CanvasTextPreview { text: normalized },
    }
}

fn canvas_file_placeholder(file: &str) -> String {
    if is_remote_image_target(file) {
        return file.to_owned();
    }

    Path::new(file)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(file)
        .to_owned()
}

fn canvas_node_center(node: &basscript_core::CanvasNode) -> Vec2 {
    let size = canvas_node_size(node.width, node.height);
    Vec2::new(node.x + size.x * 0.5, node.y + size.y * 0.5)
}

fn canvas_edge_centers(
    canvas: &CanvasDocument,
    edge: &basscript_core::CanvasEdge,
) -> Option<(Vec2, Vec2)> {
    let from = canvas.nodes.iter().find(|node| node.id == edge.from_node)?;
    let to = canvas.nodes.iter().find(|node| node.id == edge.to_node)?;
    Some((canvas_node_center(from), canvas_node_center(to)))
}

fn canvas_edge_segment_rect(
    from_center: Vec2,
    to_center: Vec2,
    segment: CanvasEdgeSegment,
    pan: Vec2,
    zoom: f32,
) -> Option<(f32, f32, f32, f32)> {
    let from = (from_center - pan) * zoom;
    let to = (to_center - pan) * zoom;
    let thickness = (2.0 * zoom).clamp(1.0, 4.0);

    match segment {
        CanvasEdgeSegment::Horizontal => {
            let width = (to.x - from.x).abs();
            (width > f32::EPSILON).then(|| {
                (
                    from.x.min(to.x),
                    from.y - thickness * 0.5,
                    width.max(thickness),
                    thickness,
                )
            })
        }
        CanvasEdgeSegment::Vertical => {
            let height = (to.y - from.y).abs();
            (height > f32::EPSILON).then(|| {
                (
                    to.x - thickness * 0.5,
                    from.y.min(to.y),
                    thickness,
                    height.max(thickness),
                )
            })
        }
    }
}
