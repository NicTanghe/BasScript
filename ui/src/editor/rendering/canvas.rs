pub(crate) const CANVAS_TEXT_SELECTION_RECT_CAPACITY: usize = 64;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CanvasRenderedNode {
    pub(crate) index: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CanvasRenderedNodeText {
    pub(crate) index: usize,
}

#[derive(Component)]
pub(crate) enum CanvasRenderedTextColor {
    Action,
    Styled(ProcessedLineRenderOverride),
    Link(String),
}

impl CanvasRenderedTextColor {
    fn color(&self, state: &EditorState) -> Color {
        match self {
            Self::Action => state.text_action_color,
            Self::Styled(style) => {
                processed_line_style_for_state(state, &style.kind, style.markdown_heading_level)
                    .color
            }
            Self::Link(target) => match state.processed_link_color_mode {
                ProcessedLinkColorMode::Colored => {
                    state.processed_link_color_for_target(Some(target))
                }
                ProcessedLinkColorMode::Hovered | ProcessedLinkColorMode::Plain => {
                    state.text_action_color
                }
            },
        }
    }
}

fn canvas_node_background(state: &EditorState, kind: &CanvasNodeKind) -> Color {
    match kind {
        CanvasNodeKind::Group { .. } => state.ui_colors.color(UiColor::PanelBackground),
        _ => state.paper_bg_color.with_alpha(1.0),
    }
}

fn canvas_node_border(state: &EditorState, id: &str) -> Color {
    state
        .ui_colors
        .color(if state.canvas_editing_node_id.as_deref() == Some(id) {
            UiColor::ActiveBackground
        } else {
            UiColor::Border
        })
}

// Theme changes must update existing nodes without rebuilding the document or
// disrupting text editing, selection, pan, or zoom.
pub(crate) fn sync_canvas_theme(
    state: Res<EditorState>,
    mut nodes: Query<
        (&CanvasRenderedNode, &mut BackgroundColor, &mut BorderColor),
        Without<CanvasRenderedEdge>,
    >,
    mut edges: Query<&mut BackgroundColor, (With<CanvasRenderedEdge>, Without<CanvasRenderedNode>)>,
    mut texts: Query<(&CanvasRenderedTextColor, &mut TextColor)>,
) {
    let Some(canvas) = state
        .canvas_document
        .as_ref()
        .filter(|_| state.document_format == DocumentFormat::Canvas)
    else {
        return;
    };
    for (rendered, mut background, mut border) in &mut nodes {
        let Some(node) = canvas.nodes.get(rendered.index) else {
            continue;
        };
        background.set_if_neq(BackgroundColor(canvas_node_background(&state, &node.kind)));
        border.set_if_neq(BorderColor::all(canvas_node_border(&state, &node.id)));
    }
    for mut edge in &mut edges {
        edge.set_if_neq(BackgroundColor(state.ui_colors.color(UiColor::Border)));
    }
    for (role, mut color) in &mut texts {
        color.set_if_neq(TextColor(role.color(&state)));
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) struct CanvasRenderedTextStyle {
    pub(crate) font_scale: f32,
    pub(crate) line_height_scale: f32,
}

impl CanvasRenderedTextStyle {
    pub(crate) const NORMAL: Self = Self {
        font_scale: 1.0,
        line_height_scale: 1.0,
    };
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CanvasRenderedTextSelection {
    pub(crate) node_index: usize,
    pub(crate) rect_index: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CanvasRenderedTextCaret {
    pub(crate) node_index: usize,
}

/// A view of Bevy's shaped text buffer in logical UI coordinates.
///
/// `TextLayoutInfo::glyphs` is renderer-facing in Bevy 0.19 and no longer
/// preserves source byte ranges. `ComputedTextBlock` retains the shaped text
/// buffer, including visual clusters, cursor geometry, selection geometry,
/// bidi ordering, and wrapped-line boundaries.
#[derive(Clone, Copy)]
pub(crate) struct CanvasTextLayout<'a> {
    pub(crate) block: &'a bevy::text::ComputedTextBlock,
    pub(crate) inverse_scale: f32,
}

#[derive(Component, Clone, Debug)]
pub(crate) struct CanvasRenderedImage {
    pub(crate) handle: Handle<Image>,
}

#[derive(Component, Clone, Debug)]
pub(crate) struct CanvasRenderedImageError {
    pub(crate) handle: Handle<Image>,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CanvasRenderedEdge {
    pub(crate) index: usize,
    pub(crate) segment: CanvasEdgeSegment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CanvasEdgeSegment {
    Horizontal,
    Vertical,
}

pub(crate) fn maybe_center_canvas_view_after_layout(
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

pub(crate) fn sync_canvas_board(
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
    mut text_query: Query<(&CanvasRenderedTextStyle, &mut TextFont, &mut LineHeight)>,
    mut text_node_query: Query<
        (&CanvasRenderedNodeText, &mut Node),
        (Without<CanvasRenderedNode>, Without<CanvasRenderedEdge>),
    >,
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
                    spawn_canvas_edge(parent, index, &state);
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
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let size = canvas_node_size(canvas_node.width, canvas_node.height);
        node.left = px((canvas_node.x - state.canvas_pan.x) * zoom);
        node.top = px((canvas_node.y - state.canvas_pan.y) * zoom);
        node.width = px(size.x * zoom);
        node.height = px(size.y * zoom);
        visibility.set_if_neq(Visibility::Visible);
    }

    for (rendered_edge, mut node, mut visibility) in edge_query.iter_mut() {
        let Some(edge) = canvas.edges.get(rendered_edge.index) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let Some((from_center, to_center)) = canvas_edge_centers(canvas, edge) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let Some((left, top, width, height)) = canvas_edge_segment_rect(
            from_center,
            to_center,
            rendered_edge.segment,
            state.canvas_pan,
            zoom,
        ) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };

        node.left = px(left);
        node.top = px(top);
        node.width = px(width);
        node.height = px(height);
        visibility.set_if_neq(Visibility::Visible);
    }

    for (style, mut text_font, mut line_height) in text_query.iter_mut() {
        let mut next_font = text_font.clone();
        let mut next_line_height = *line_height;
        update_canvas_text_metrics(zoom, style, &mut next_font, &mut next_line_height);
        text_font.set_if_neq(next_font);
        line_height.set_if_neq(next_line_height);
    }
    for (_, mut text_node) in text_node_query.iter_mut() {
        let mut next_node = text_node.clone();
        update_canvas_text_padding(zoom, &mut next_node);
        text_node.set_if_neq(next_node);
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

pub(crate) fn sync_canvas_text_overlays(
    text_layout_query: Query<(
        &CanvasRenderedNodeText,
        &bevy::text::ComputedTextBlock,
        &ComputedNode,
    )>,
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
    blink: Res<CaretBlinkState>,
) {
    if state.document_format != DocumentFormat::Canvas {
        for (_, mut node, _, mut visibility) in text_selection_query.iter_mut() {
            node.width = px(0.0);
            node.height = px(0.0);
            visibility.set_if_neq(Visibility::Hidden);
        }
        for (_, mut node, _, mut visibility, mut transform) in text_caret_query.iter_mut() {
            node.width = px(0.0);
            node.height = px(0.0);
            transform.scale = Vec2::ONE;
            transform.translation = Val2::ZERO;
            visibility.set_if_neq(Visibility::Hidden);
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
    render_canvas_text_carets(
        &mut text_caret_query,
        &text_layout_query,
        canvas,
        &state,
        zoom,
        blink.visible,
    );
}

pub(crate) fn spawn_canvas_edge(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    state: &EditorState,
) {
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
            BackgroundColor(state.ui_colors.color(UiColor::Border)),
            ZIndex(10),
            CanvasRenderedEdge { index, segment },
        ));
    }
}

pub(crate) fn spawn_canvas_node(
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
    let node_color = canvas_node_background(state, &node.kind);
    let active_text_node = state.canvas_editing_node_id.as_deref() == Some(node.id.as_str());
    let border_color = canvas_node_border(state, &node.id);

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
            // Group surfaces use the shared panel color, which can be opaque.
            // Keep them behind cards and connectors regardless of file order.
            ZIndex(if matches!(node.kind, CanvasNodeKind::Group { .. }) {
                0
            } else {
                20 + index as i32
            }),
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
                let color_role = CanvasRenderedTextColor::Link(url.clone());
                node_parent.spawn((
                    Text::new(url.clone()),
                    text_font_for_variant(
                        fonts,
                        FontVariant::Regular,
                        DocumentFormat::Canvas,
                        canvas_text_font_size(zoom),
                    ),
                    LineHeight::Px(canvas_text_line_height(zoom)),
                    TextColor(color_role.color(state)),
                    color_role,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(canvas_text_padding_x(zoom)),
                        top: px(canvas_text_padding_y(zoom)),
                        right: px(canvas_text_padding_x(zoom)),
                        bottom: px(canvas_text_padding_y(zoom)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    CanvasRenderedNodeText { index },
                    CanvasRenderedTextStyle::NORMAL,
                ));
            }
            CanvasNodeKind::Group { label } => {
                node_parent.spawn((
                    Text::new(label.clone().unwrap_or_default()),
                    text_font_for_variant(
                        fonts,
                        FontVariant::Bold,
                        DocumentFormat::Canvas,
                        canvas_text_font_size(zoom),
                    ),
                    LineHeight::Px(canvas_text_line_height(zoom)),
                    ThemedText::Muted,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(canvas_text_padding_x(zoom)),
                        top: px(canvas_text_padding_y(zoom)),
                        right: px(canvas_text_padding_x(zoom)),
                        bottom: px(canvas_text_padding_y(zoom)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    CanvasRenderedNodeText { index },
                    CanvasRenderedTextStyle::NORMAL,
                ));
            }
            CanvasNodeKind::Unknown { node_type } => {
                node_parent.spawn((
                    Text::new(format!("Unsupported canvas node: {node_type}")),
                    text_font_for_variant(
                        fonts,
                        FontVariant::Regular,
                        DocumentFormat::Canvas,
                        canvas_text_font_size(zoom),
                    ),
                    LineHeight::Px(canvas_text_line_height(zoom)),
                    ThemedText::Muted,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(canvas_text_padding_x(zoom)),
                        top: px(canvas_text_padding_y(zoom)),
                        right: px(canvas_text_padding_x(zoom)),
                        bottom: px(canvas_text_padding_y(zoom)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    CanvasRenderedNodeText { index },
                    CanvasRenderedTextStyle::NORMAL,
                ));
            }
        });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CanvasTextRenderMode {
    Rendered,
    Plain,
    PlainInteractive,
}

pub(crate) fn spawn_canvas_text_preview(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    text: &str,
    mode: CanvasTextRenderMode,
    state: &EditorState,
    fonts: &EditorFonts,
    zoom: f32,
) {
    if mode == CanvasTextRenderMode::PlainInteractive {
        spawn_canvas_text_selection_rects(parent, index);
        spawn_canvas_text_caret(parent, index);
    }

    if mode == CanvasTextRenderMode::Rendered {
        spawn_canvas_rendered_text_preview(parent, index, text, state, fonts, zoom);
        return;
    }

    let preview = canvas_text_preview(text, mode, state);
    parent.spawn((
        Text::new(preview.text),
        TextFont {
            font: fonts.regular.clone().into(),
            font_size: FontSize::Px(canvas_text_font_size(zoom)),
            ..default()
        },
        LineHeight::Px(canvas_text_line_height(zoom)),
        TextColor(state.text_action_color),
        CanvasRenderedTextColor::Action,
        Node {
            position_type: PositionType::Absolute,
            left: px(canvas_text_padding_x(zoom)),
            top: px(canvas_text_padding_y(zoom)),
            right: px(canvas_text_padding_x(zoom)),
            bottom: px(canvas_text_padding_y(zoom)),
            overflow: Overflow::clip(),
            ..default()
        },
        ZIndex(1),
        CanvasRenderedNodeText { index },
        CanvasRenderedTextStyle::NORMAL,
    ));
}

pub(crate) fn spawn_canvas_rendered_text_preview(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    text: &str,
    state: &EditorState,
    fonts: &EditorFonts,
    zoom: f32,
) {
    let font_size = canvas_text_font_size(zoom);
    let line_height = canvas_text_line_height(zoom);
    parent
        .spawn((
            Text::new(""),
            text_font_for_variant(
                fonts,
                FontVariant::Regular,
                DocumentFormat::Canvas,
                font_size,
            ),
            LineHeight::Px(line_height),
            TextColor(state.text_action_color),
            CanvasRenderedTextColor::Action,
            Node {
                position_type: PositionType::Absolute,
                left: px(canvas_text_padding_x(zoom)),
                top: px(canvas_text_padding_y(zoom)),
                right: px(canvas_text_padding_x(zoom)),
                bottom: px(canvas_text_padding_y(zoom)),
                overflow: Overflow::clip(),
                ..default()
            },
            ZIndex(1),
            CanvasRenderedNodeText { index },
            CanvasRenderedTextStyle::NORMAL,
        ))
        .with_children(|text_parent| {
            for span in canvas_rendered_text_spans(text, state) {
                text_parent.spawn((
                    TextSpan::new(span.text),
                    text_font_for_variant(
                        fonts,
                        span.style.font_variant,
                        DocumentFormat::Canvas,
                        font_size * span.style.font_scale,
                    ),
                    LineHeight::Px(line_height * span.style.line_height_scale),
                    TextColor(span.style.color),
                    span.render_override.map_or(
                        CanvasRenderedTextColor::Action,
                        CanvasRenderedTextColor::Styled,
                    ),
                    CanvasRenderedTextStyle {
                        font_scale: span.style.font_scale,
                        line_height_scale: span.style.line_height_scale,
                    },
                ));
            }
        });
}

#[derive(Clone, Debug)]
pub(crate) struct CanvasTextPreviewSpan {
    pub(crate) text: String,
    pub(crate) style: LineRenderStyle,
    pub(crate) render_override: Option<ProcessedLineRenderOverride>,
}

#[derive(Default)]
pub(crate) struct CanvasTextPreview {
    pub(crate) text: String,
}

pub(crate) fn spawn_canvas_text_selection_rects(
    parent: &mut ChildSpawnerCommands,
    node_index: usize,
) {
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

pub(crate) fn spawn_canvas_text_caret(parent: &mut ChildSpawnerCommands, node_index: usize) {
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
        BackgroundColor(Color::BLACK),
        Visibility::Hidden,
        ZIndex(2),
        CanvasRenderedTextCaret { node_index },
    ));
}

pub(crate) fn render_canvas_text_selection_rects(
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
    text_layout_query: &Query<(
        &CanvasRenderedNodeText,
        &bevy::text::ComputedTextBlock,
        &ComputedNode,
    )>,
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
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        }

        let (left, top, width, height) = rects[selection.rect_index];
        node.left = px(left);
        node.top = px(top);
        node.width = px(width);
        node.height = px(height);
        color.0 = state.selection_bg_color;
        visibility.set_if_neq(Visibility::Visible);
    }
}

pub(crate) fn render_canvas_text_carets(
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
    text_layout_query: &Query<(
        &CanvasRenderedNodeText,
        &bevy::text::ComputedTextBlock,
        &ComputedNode,
    )>,
    canvas: &CanvasDocument,
    state: &EditorState,
    zoom: f32,
    caret_visible: bool,
) {
    let active_node_index = canvas_active_text_node_index(canvas, state);
    let caret_rect = active_node_index
        .filter(|_| caret_visible)
        .and_then(|node_index| {
            canvas_text_caret_rect(canvas, node_index, state, text_layout_query, zoom)
        });

    for (caret, mut node, mut color, mut visibility, mut transform) in caret_query.iter_mut() {
        if Some(caret.node_index) != active_node_index || caret_rect.is_none() {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        }

        let (left, top, width, height) = caret_rect.unwrap();
        node.left = px(left);
        node.top = px(top);
        node.width = px(width);
        node.height = px(height);
        color.set_if_neq(BackgroundColor(caret_color_for_state(
            state,
            PanelKind::Processed,
        )));
        transform.scale = Vec2::ONE;
        transform.translation = Val2::ZERO;
        visibility.set_if_neq(Visibility::Visible);
    }
}

pub(crate) fn canvas_active_text_node_index(
    canvas: &CanvasDocument,
    state: &EditorState,
) -> Option<usize> {
    let node_id = state.canvas_editing_node_id.as_deref()?;
    canvas
        .nodes
        .iter()
        .position(|node| node.id == node_id && matches!(node.kind, CanvasNodeKind::Text { .. }))
}

pub(crate) fn canvas_text_selection_rects(
    canvas: &CanvasDocument,
    node_index: usize,
    state: &EditorState,
    text_layout_query: &Query<(
        &CanvasRenderedNodeText,
        &bevy::text::ComputedTextBlock,
        &ComputedNode,
    )>,
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

    let Some(layout) = canvas_text_layout_for_node(text_layout_query, node_index)
        .filter(|layout| canvas_text_layout_matches_document(*layout, &document))
    else {
        return canvas_text_selection_rects_fallback(&document, start, end, zoom);
    };

    let buffer = layout.block.buffer();
    let start_byte = canvas_text_document_byte_offset(&document, start);
    let end_byte = canvas_text_document_byte_offset(&document, end);
    let selection = parley::Selection::new(
        parley::Cursor::from_byte_index(buffer, start_byte, parley::Affinity::Downstream),
        parley::Cursor::from_byte_index(buffer, end_byte, parley::Affinity::Upstream),
    );
    let inverse_scale = layout.inverse_scale.max(f32::EPSILON);
    let rects = selection
        .geometry(buffer)
        .into_iter()
        .take(CANVAS_TEXT_SELECTION_RECT_CAPACITY)
        .filter_map(|(rect, _)| {
            let left = rect.x0 as f32 * inverse_scale;
            let top = rect.y0 as f32 * inverse_scale;
            let width = rect.width() as f32 * inverse_scale;
            let height = rect.height() as f32 * inverse_scale;
            (left.is_finite() && top.is_finite() && width.is_finite() && height.is_finite())
                .then_some((
                    canvas_text_padding_x(zoom) + left,
                    canvas_text_padding_y(zoom) + top,
                    width.max(1.0),
                    height.max(1.0),
                ))
        })
        .collect::<Vec<_>>();

    if rects.is_empty() {
        canvas_text_selection_rects_fallback(&document, start, end, zoom)
    } else {
        rects
    }
}

pub(crate) fn canvas_text_selection_rects_fallback(
    document: &Document,
    start: Position,
    end: Position,
    zoom: f32,
) -> Vec<(f32, f32, f32, f32)> {
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

        let left_x = line_start as f32 * fallback_char_width;
        let right_x = line_end as f32 * fallback_char_width;
        let line_top = line as f32 * fallback_line_height;
        rects.push((
            canvas_text_padding_x(zoom) + left_x.min(right_x),
            canvas_text_padding_y(zoom)
                + (line_top + caret_vertical_offset(fallback_line_height)).max(0.0),
            (right_x - left_x).abs().max(1.0),
            fallback_line_height.max(1.0),
        ));
    }

    rects
}

pub(crate) fn canvas_text_caret_rect(
    canvas: &CanvasDocument,
    node_index: usize,
    state: &EditorState,
    text_layout_query: &Query<(
        &CanvasRenderedNodeText,
        &bevy::text::ComputedTextBlock,
        &ComputedNode,
    )>,
    zoom: f32,
) -> Option<(f32, f32, f32, f32)> {
    let node = canvas.nodes.get(node_index)?;
    let CanvasNodeKind::Text { text } = &node.kind else {
        return None;
    };
    let document = Document::from_text(text);
    let cursor = document.clamp_position(state.canvas_text_cursor.position);
    let fallback_char_width = canvas_text_char_width(zoom);
    let fallback_line_height = canvas_text_line_height(zoom);

    if let Some(layout) = canvas_text_layout_for_node(text_layout_query, node_index)
        .filter(|layout| canvas_text_layout_matches_document(*layout, &document))
    {
        let inverse_scale = layout.inverse_scale.max(f32::EPSILON);
        let requested_width = caret_width_for_state(state, fallback_char_width);
        let buffer = layout.block.buffer();
        let byte_index = canvas_text_document_byte_offset(&document, cursor);
        let cursor =
            parley::Cursor::from_byte_index(buffer, byte_index, parley::Affinity::Downstream);
        let rect = cursor.geometry(buffer, requested_width / inverse_scale);
        let left = rect.x0 as f32 * inverse_scale;
        let top = rect.y0 as f32 * inverse_scale;
        let width = rect.width() as f32 * inverse_scale;
        let height = rect.height() as f32 * inverse_scale;
        if left.is_finite() && top.is_finite() && width.is_finite() && height.is_finite() {
            return Some((
                canvas_text_padding_x(zoom) + (left + caret_x_offset_for_state(state)).max(0.0),
                canvas_text_padding_y(zoom) + top.max(0.0),
                width.max(1.0),
                height.max(1.0),
            ));
        }
    }

    Some((
        canvas_text_padding_x(zoom)
            + (cursor.column as f32 * fallback_char_width + caret_x_offset_for_state(state))
                .max(0.0),
        canvas_text_padding_y(zoom)
            + (cursor.line as f32 * fallback_line_height
                + caret_vertical_offset(fallback_line_height))
            .max(0.0),
        caret_width_for_state(state, fallback_char_width),
        fallback_line_height.max(1.0),
    ))
}

pub(crate) fn canvas_text_layout_for_node<'a>(
    text_layout_query: &'a Query<(
        &CanvasRenderedNodeText,
        &bevy::text::ComputedTextBlock,
        &ComputedNode,
    )>,
    node_index: usize,
) -> Option<CanvasTextLayout<'a>> {
    text_layout_query
        .iter()
        .find(|(node_text, _, _)| node_text.index == node_index)
        .map(|(_, block, computed)| CanvasTextLayout {
            block,
            inverse_scale: computed.inverse_scale_factor(),
        })
}

pub(crate) fn canvas_text_layout_matches_document(
    layout: CanvasTextLayout<'_>,
    document: &Document,
) -> bool {
    if layout.block.needs_rerender(false, false) {
        return false;
    }

    let buffer_len = layout
        .block
        .buffer()
        .lines()
        .map(|line| line.text_range().end)
        .max()
        .unwrap_or_default();
    buffer_len == document.to_text().len()
}

pub(crate) fn canvas_text_document_byte_offset(document: &Document, position: Position) -> usize {
    let position = document.clamp_position(position);
    let preceding_bytes = (0..position.line)
        .map(|line| document.line(line).map_or(0, str::len).saturating_add(1))
        .sum::<usize>();
    let line_text = document.line(position.line).unwrap_or_default();
    preceding_bytes + char_to_byte_index(line_text, position.column)
}

pub(crate) fn canvas_text_position_from_document_byte(
    document: &Document,
    byte_index: usize,
) -> Position {
    let mut line_start = 0usize;
    for line in 0..document.line_count() {
        let line_text = document.line(line).unwrap_or_default();
        let line_end = line_start.saturating_add(line_text.len());
        if byte_index <= line_end {
            let line_byte = byte_index.saturating_sub(line_start).min(line_text.len());
            return Position {
                line,
                column: byte_to_char_index(line_text, line_byte),
            };
        }

        if line + 1 == document.line_count() {
            return Position {
                line,
                column: line_text.chars().count(),
            };
        }
        line_start = line_end.saturating_add(1);
    }

    Position::default()
}

pub(crate) fn canvas_text_position_from_layout(
    document: &Document,
    layout: CanvasTextLayout<'_>,
    x: f32,
    y: f32,
) -> Option<Position> {
    canvas_text_layout_matches_document(layout, document).then(|| {
        let inverse_scale = layout.inverse_scale.max(f32::EPSILON);
        let cursor =
            parley::Cursor::from_point(layout.block.buffer(), x / inverse_scale, y / inverse_scale);
        canvas_text_position_from_document_byte(document, cursor.index())
    })
}

pub(crate) fn spawn_canvas_image_or_placeholder(
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
                        font: fonts.markdown_regular.clone().into(),
                        font_size: FontSize::Px(canvas_text_font_size(zoom)),
                        ..default()
                    },
                    LineHeight::Px(canvas_text_line_height(zoom)),
                    ThemedText::Muted,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(canvas_text_padding_x(zoom)),
                        top: px(canvas_text_padding_y(zoom)),
                        right: px(canvas_text_padding_x(zoom)),
                        bottom: px(canvas_text_padding_y(zoom)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    Visibility::Hidden,
                    CanvasRenderedNodeText { index },
                    CanvasRenderedTextStyle::NORMAL,
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

pub(crate) fn spawn_canvas_image_error(
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
            font: fonts.markdown_regular.clone().into(),
            font_size: FontSize::Px(canvas_text_font_size(zoom)),
            ..default()
        },
        LineHeight::Px(canvas_text_line_height(zoom)),
        ThemedText::Muted,
        Node {
            position_type: PositionType::Absolute,
            left: px(canvas_text_padding_x(zoom)),
            top: px(canvas_text_padding_y(zoom)),
            right: px(canvas_text_padding_x(zoom)),
            bottom: px(canvas_text_padding_y(zoom)),
            overflow: Overflow::clip(),
            ..default()
        },
        CanvasRenderedNodeText { index },
        CanvasRenderedTextStyle::NORMAL,
    ));
}

pub(crate) fn update_canvas_text_metrics(
    zoom: f32,
    style: &CanvasRenderedTextStyle,
    text_font: &mut TextFont,
    line_height: &mut LineHeight,
) {
    text_font.font_size = FontSize::Px(canvas_text_font_size(zoom) * style.font_scale);
    *line_height = LineHeight::Px(canvas_text_line_height(zoom) * style.line_height_scale);
}

pub(crate) fn update_canvas_text_padding(zoom: f32, node: &mut Node) {
    let padding_x = px(canvas_text_padding_x(zoom));
    let padding_y = px(canvas_text_padding_y(zoom));
    node.left = padding_x;
    node.top = padding_y;
    node.right = padding_x;
    node.bottom = padding_y;
}

pub(crate) fn canvas_image_load_failed(asset_server: &AssetServer, handle: &Handle<Image>) -> bool {
    matches!(
        asset_server.get_load_state(handle),
        Some(LoadState::Failed(_))
    )
}

pub(crate) fn canvas_image_error_text(message: &str, target: &str) -> String {
    format!("{message}\n{}", canvas_file_placeholder(target))
}

pub(crate) fn canvas_first_image_target(text: &str) -> Option<String> {
    canvas_first_markdown_image_embed(text)
        .map(|embed| embed.target)
        .or_else(|| canvas_first_html_image_src(text))
        .or_else(|| canvas_first_obsidian_image_target(text))
        .or_else(|| canvas_first_remote_image_url(text))
        .and_then(|target| canvas_clean_image_target(&target))
}

pub(crate) fn remote_image_target_seems_loadable(target: &str) -> bool {
    let trimmed = target.trim();
    if !is_remote_image_target(trimmed) {
        return true;
    }
    let without_fragment = trimmed.split('#').next().unwrap_or(trimmed);
    let without_query = without_fragment
        .split('?')
        .next()
        .unwrap_or(without_fragment);
    let lower = without_query.to_ascii_lowercase();
    matches!(
        Path::new(&lower).extension().and_then(|ext| ext.to_str()),
        Some("png" | "jpg" | "jpeg" | "webp" | "bmp")
    )
}

pub(crate) fn canvas_first_markdown_image_embed(text: &str) -> Option<ImageEmbed> {
    parse_document_with_format(&Document::from_text(text), DocumentFormat::Markdown)
        .into_iter()
        .flat_map(|line| line.image_embeds)
        .next()
}

pub(crate) fn canvas_first_html_image_src(text: &str) -> Option<String> {
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

pub(crate) fn canvas_first_obsidian_image_target(text: &str) -> Option<String> {
    let start = text.find("![[")? + 3;
    let rest = text.get(start..)?;
    let end = rest
        .find("]]")
        .or_else(|| rest.find(']'))
        .unwrap_or(rest.len());
    let target = rest.get(..end)?;
    Some(target.split('|').next().unwrap_or(target).trim().to_owned())
}

pub(crate) fn canvas_first_remote_image_url(text: &str) -> Option<String> {
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

pub(crate) fn canvas_clean_image_target(target: &str) -> Option<String> {
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

pub(crate) fn html_attr_value(tag: &str, attr: &str) -> Option<String> {
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

pub(crate) fn skip_ascii_whitespace(input: &str, mut index: usize) -> usize {
    while input
        .as_bytes()
        .get(index)
        .is_some_and(u8::is_ascii_whitespace)
    {
        index += 1;
    }
    index
}

pub(crate) fn canvas_text_render_mode(
    state: &EditorState,
    active_text_node: bool,
) -> CanvasTextRenderMode {
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

pub(crate) fn canvas_text_interaction_active(state: &EditorState) -> bool {
    state.canvas_editing_node_id.is_some()
        && (!state.vim_enabled
            || matches!(
                state.vim_mode,
                VimMode::Normal | VimMode::Insert | VimMode::VisualChar | VimMode::VisualLine
            ))
}

pub(crate) fn canvas_text_preview(
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
        CanvasTextRenderMode::Plain => CanvasTextPreview { text: normalized },
        CanvasTextRenderMode::PlainInteractive => CanvasTextPreview { text: normalized },
    }
}

pub(crate) fn canvas_rendered_text_spans(
    text: &str,
    state: &EditorState,
) -> Vec<CanvasTextPreviewSpan> {
    let normalized = text.replace("\r\n", "\n");
    let lines = normalized.split('\n').collect::<Vec<_>>();
    if lines.is_empty() {
        return vec![CanvasTextPreviewSpan {
            text: String::new(),
            style: default_line_render_style_for_state(state),
            render_override: None,
        }];
    }

    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let render_override = markdown_render_override_for_raw(line);
            let style = render_override
                .as_ref()
                .map(|override_style| {
                    processed_line_style_for_state(
                        state,
                        &override_style.kind,
                        override_style.markdown_heading_level,
                    )
                })
                .unwrap_or_else(|| default_line_render_style_for_state(state));
            let mut rendered = render_override
                .as_ref()
                .and_then(|override_style| {
                    markdown_visual_text_for_kind(
                        line,
                        &override_style.kind,
                        override_style.markdown_heading_level,
                    )
                })
                .map(|(_, rendered, _)| rendered)
                .unwrap_or_else(|| (*line).to_owned());

            if index + 1 < lines.len() {
                rendered.push('\n');
            }

            CanvasTextPreviewSpan {
                text: rendered,
                style,
                render_override,
            }
        })
        .collect()
}

pub(crate) fn canvas_file_placeholder(file: &str) -> String {
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

pub(crate) fn canvas_node_center(node: &basscript_core::CanvasNode) -> Vec2 {
    let size = canvas_node_size(node.width, node.height);
    Vec2::new(node.x + size.x * 0.5, node.y + size.y * 0.5)
}

pub(crate) fn canvas_edge_centers(
    canvas: &CanvasDocument,
    edge: &basscript_core::CanvasEdge,
) -> Option<(Vec2, Vec2)> {
    let from = canvas.nodes.iter().find(|node| node.id == edge.from_node)?;
    let to = canvas.nodes.iter().find(|node| node.id == edge.to_node)?;
    Some((canvas_node_center(from), canvas_node_center(to)))
}

pub(crate) fn canvas_edge_segment_rect(
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
#[allow(unused_imports)]
use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::{IntoSystem, System};
    use bevy::prelude::World;

    fn theme_test_app(editing: bool) -> App {
        let mut app = App::new();
        app.add_plugins((bevy::app::TaskPoolPlugin::default(), AssetPlugin::default()))
            .init_asset::<Image>()
            .init_resource::<EditorState>()
            .init_resource::<EditorImageCache>()
            .init_resource::<CaretBlinkState>()
            .insert_resource(EditorFonts {
                regular: default(),
                bold: default(),
                italic: default(),
                bold_italic: default(),
                markdown_regular: default(),
                markdown_bold: default(),
                markdown_italic: default(),
                markdown_bold_italic: default(),
            })
            .add_systems(
                Update,
                (sync_canvas_board, sync_canvas_text_overlays).chain(),
            )
            .add_systems(PostUpdate, (sync_canvas_theme, sync_theme_widgets));
        app.world_mut().spawn((
            PanelCanvas {
                kind: PanelKind::Processed,
            },
            Node::default(),
        ));
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.document_format = DocumentFormat::Canvas;
        state.document = Document::from_text(
            r##"{"nodes":[
            {"id":"text","type":"text","text":"# Heading\nBody\n> Quote\n---\n```","x":0,"y":0,"width":300,"height":300},
            {"id":"group","type":"group","label":"Group","x":0,"y":0,"width":500,"height":500},
            {"id":"link","type":"link","url":"https://example.com","x":400,"y":0,"width":300,"height":100},
            {"id":"file","type":"file","file":"missing-test-image.png","x":400,"y":200,"width":300,"height":100}
        ],"edges":[{"id":"edge","fromNode":"text","toNode":"link"}]}"##,
        );
        state.reparse();
        state.display_mode = if editing {
            DisplayMode::Plain
        } else {
            DisplayMode::Processed
        };
        state.processed_link_color_mode = ProcessedLinkColorMode::Colored;
        state.vim_enabled = false;
        state.canvas_editing_node_id = editing.then(|| "text".to_owned());
        state.canvas_text_cursor = Cursor {
            position: Position { line: 1, column: 4 },
            preferred_column: 4,
        };
        state.canvas_text_selection_anchor = Some(Position { line: 1, column: 0 });
        drop(state);
        app
    }

    #[test]
    fn canvas_nodes_follow_live_theme_changes_without_rebuilding_or_losing_edit_state() {
        for editing in [false, true] {
            let mut app = theme_test_app(editing);
            app.update();
            let nodes = {
                let world = app.world_mut();
                world
                    .query_filtered::<Entity, With<CanvasRenderedNode>>()
                    .iter(world)
                    .collect::<Vec<_>>()
            };
            assert_eq!(nodes.len(), 4);
            let version = app.world().resource::<EditorState>().canvas_version;
            let cursor = app.world().resource::<EditorState>().canvas_text_cursor;
            let mut custom = ThemeSettings::dark();
            custom.paper_background = Vec4::new(0.2, 0.1, 0.15, 0.25);
            custom.text_action = Vec4::new(0.9, 0.7, 0.8, 1.0);
            custom.text_markdown_heading = Vec4::new(0.8, 0.2, 0.6, 1.0);
            custom.text_markdown_quote = Vec4::new(0.4, 0.8, 0.6, 1.0);
            custom.text_markdown_rule = Vec4::new(0.5, 0.6, 0.8, 1.0);
            custom.text_markdown_code = Vec4::new(0.9, 0.5, 0.1, 1.0);
            custom.link_fallback = Vec4::new(0.1, 0.6, 0.9, 1.0);
            for theme in [ThemeSettings::default(), ThemeSettings::dark(), custom] {
                apply_theme_to_state(&mut app.world_mut().resource_mut::<EditorState>(), &theme);
                app.update();
                let world = app.world_mut();
                assert_eq!(world.resource::<EditorState>().canvas_version, version);
                assert_eq!(world.resource::<EditorState>().canvas_text_cursor, cursor);
                for &entity in &nodes {
                    let node = world
                        .get::<CanvasRenderedNode>(entity)
                        .expect("theme changes should reuse the node");
                    assert_eq!(
                        world.get::<BackgroundColor>(entity).unwrap().0,
                        if node.index == 1 {
                            theme.ui_colors.color(UiColor::PanelBackground)
                        } else {
                            theme.paper_background_color().with_alpha(1.0)
                        }
                    );
                    assert_eq!(
                        *world.get::<BorderColor>(entity).unwrap(),
                        BorderColor::all(theme.ui_colors.color(if editing && node.index == 0 {
                            UiColor::ActiveBackground
                        } else {
                            UiColor::Border
                        }))
                    );
                }
                for color in world
                    .query_filtered::<&BackgroundColor, With<CanvasRenderedEdge>>()
                    .iter(world)
                {
                    assert_eq!(color.0, theme.ui_colors.color(UiColor::Border));
                }
                for (role, color) in world.query::<(&ThemedText, &TextColor)>().iter(world) {
                    assert!(matches!(role, ThemedText::Muted));
                    assert_eq!(color.0, theme.text_muted_color());
                }
                for (text, color) in world.query::<(&Text, &TextColor)>().iter(world) {
                    if text.0 == "https://example.com" {
                        assert_eq!(color.0, color_from_rgba(theme.link_fallback));
                    }
                }
                if editing {
                    for (_, color, visibility) in world
                        .query::<(&CanvasRenderedTextCaret, &BackgroundColor, &Visibility)>()
                        .iter(world)
                    {
                        assert_eq!(*visibility, Visibility::Visible);
                        assert_eq!(
                            color.0,
                            if theme.name == "Default" {
                                Color::BLACK
                            } else {
                                Color::WHITE
                            }
                        );
                    }
                    let selection_colors = world
                        .query::<(&CanvasRenderedTextSelection, &BackgroundColor, &Visibility)>()
                        .iter(world)
                        .filter(|(_, _, visibility)| **visibility == Visibility::Visible)
                        .map(|(_, color, _)| color.0)
                        .collect::<Vec<_>>();
                    assert_eq!(selection_colors, vec![theme.selection_background_color()]);
                } else {
                    let colors = world
                        .query::<(&TextSpan, &TextColor)>()
                        .iter(world)
                        .map(|(span, color)| (span.0.clone(), color.0))
                        .collect::<Vec<_>>();
                    for (text, expected) in [
                        ("Heading\n", theme.text_markdown_heading_color()),
                        ("Body\n", theme.text_action_color()),
                        ("Quote\n", theme.text_markdown_quote_color()),
                        (
                            "────────────────────────\n",
                            theme.text_markdown_rule_color(),
                        ),
                        ("```", theme.text_markdown_code_color()),
                    ] {
                        assert!(
                            colors.contains(&(text.to_owned(), expected)),
                            "missing themed span {text:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn canvas_board_system_queries_are_disjoint() {
        let mut world = World::new();
        let mut system = IntoSystem::into_system(sync_canvas_board);

        system.initialize(&mut world);
    }

    #[test]
    fn canvas_text_metrics_keep_rendered_span_scale_when_zoom_changes() {
        let style = CanvasRenderedTextStyle {
            font_scale: 1.5,
            line_height_scale: 1.75,
        };
        let mut text_font = TextFont::default();
        let mut line_height = LineHeight::Px(0.0);

        update_canvas_text_metrics(1.0, &style, &mut text_font, &mut line_height);
        assert_eq!(text_font.font_size, FontSize::Px(18.0));
        assert_eq!(line_height, LineHeight::Px(26.25));

        update_canvas_text_metrics(2.0, &style, &mut text_font, &mut line_height);
        assert_eq!(text_font.font_size, FontSize::Px(36.0));
        assert_eq!(line_height, LineHeight::Px(52.5));
    }

    #[test]
    fn canvas_text_and_padding_scale_linearly_at_zoom_extremes() {
        const TOLERANCE: f32 = 0.000_01;
        let base_columns =
            (CANVAS_NODE_DEFAULT_WIDTH - CANVAS_TEXT_PADDING_X * 2.0) / canvas_text_char_width(1.0);
        for zoom in [CANVAS_ZOOM_MIN, 1.0, CANVAS_ZOOM_MAX] {
            assert!((canvas_text_font_size(zoom) - FONT_SIZE * zoom).abs() < TOLERANCE);
            assert!((canvas_text_line_height(zoom) - FONT_SIZE * 1.25 * zoom).abs() < TOLERANCE);
            assert!((canvas_text_padding_x(zoom) - CANVAS_TEXT_PADDING_X * zoom).abs() < TOLERANCE);
            assert!((canvas_text_padding_y(zoom) - CANVAS_TEXT_PADDING_Y * zoom).abs() < TOLERANCE);
            let mut node = Node::default();
            update_canvas_text_padding(zoom, &mut node);
            assert_eq!(node.left, px(CANVAS_TEXT_PADDING_X * zoom));
            assert_eq!(node.top, px(CANVAS_TEXT_PADDING_Y * zoom));
            assert_eq!(node.right, px(CANVAS_TEXT_PADDING_X * zoom));
            assert_eq!(node.bottom, px(CANVAS_TEXT_PADDING_Y * zoom));

            let visible_columns = (CANVAS_NODE_DEFAULT_WIDTH * zoom
                - canvas_text_padding_x(zoom) * 2.0)
                / canvas_text_char_width(zoom);
            assert!((visible_columns - base_columns).abs() < TOLERANCE);
        }
    }
}
