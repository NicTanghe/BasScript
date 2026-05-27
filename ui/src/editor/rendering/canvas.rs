const COLOR_CANVAS_NODE_BG: Color = Color::srgb(0.96, 0.97, 0.98);
const COLOR_CANVAS_GROUP_BG: Color = Color::srgba(0.80, 0.84, 0.90, 0.28);
const COLOR_CANVAS_NODE_BORDER: Color = Color::srgba(0.08, 0.10, 0.12, 0.22);
const COLOR_CANVAS_EDGE: Color = Color::srgba(0.10, 0.12, 0.15, 0.45);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct CanvasRenderedNode {
    index: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct CanvasRenderedNodeText {
    index: usize,
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
        Without<CanvasRenderedEdge>,
    >,
    mut edge_query: Query<
        (&CanvasRenderedEdge, &mut Node, &mut Visibility),
        Without<CanvasRenderedNode>,
    >,
    mut text_query: Query<(&CanvasRenderedNodeText, &mut TextFont)>,
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

    for (_, mut text_font) in text_query.iter_mut() {
        text_font.font_size = (12.0 * zoom).clamp(7.0, 28.0);
    }
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
            BorderColor::all(COLOR_CANVAS_NODE_BORDER),
            ZIndex(20 + index as i32),
            CanvasRenderedNode { index },
        ))
        .with_children(|node_parent| match &node.kind {
            CanvasNodeKind::Text { text } => {
                let editing = state.canvas_editing_node_id.as_deref() == Some(node.id.as_str());
                if !editing {
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

                node_parent.spawn((
                    Text::new(canvas_text_preview(text, editing)),
                    TextFont {
                        font: fonts.markdown_regular.clone(),
                        font_size: (12.0 * zoom).clamp(7.0, 28.0),
                        ..default()
                    },
                    TextColor(COLOR_TEXT_MAIN),
                    Node {
                        width: percent(100.0),
                        height: percent(100.0),
                        padding: UiRect::all(px(10.0)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    CanvasRenderedNodeText { index },
                ));
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
                        font_size: (12.0 * zoom).clamp(7.0, 28.0),
                        ..default()
                    },
                    TextColor(COLOR_TEXT_MUTED),
                    Node {
                        width: percent(100.0),
                        height: percent(100.0),
                        padding: UiRect::all(px(10.0)),
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
                        font_size: (13.0 * zoom).clamp(8.0, 30.0),
                        ..default()
                    },
                    TextColor(COLOR_TEXT_MUTED),
                    Node {
                        width: percent(100.0),
                        padding: UiRect::all(px(10.0)),
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
                        font_size: (12.0 * zoom).clamp(7.0, 28.0),
                        ..default()
                    },
                    TextColor(COLOR_TEXT_MUTED),
                    Node {
                        width: percent(100.0),
                        padding: UiRect::all(px(10.0)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    CanvasRenderedNodeText { index },
                ));
            }
        });
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
    let lookup = processed_image_lookup(image_cache, state, target, asset_server, images);
    match lookup {
        ProcessedImageLookup::Loaded { handle, .. } => {
            parent.spawn((
                ImageNode::new(handle),
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
            ));
        }
        ProcessedImageLookup::Failed => {
            parent.spawn((
                Text::new(canvas_file_placeholder(target)),
                TextFont {
                    font: fonts.markdown_regular.clone(),
                    font_size: (12.0 * zoom).clamp(7.0, 28.0),
                    ..default()
                },
                TextColor(COLOR_TEXT_MUTED),
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    padding: UiRect::all(px(10.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                CanvasRenderedNodeText { index },
            ));
        }
    }
}

fn canvas_first_image_target(text: &str) -> Option<String> {
    canvas_first_markdown_image_embed(text)
        .map(|embed| embed.target)
        .or_else(|| canvas_first_html_image_src(text))
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

fn canvas_text_preview(text: &str, editing: bool) -> String {
    let mut preview = text
        .replace("\r\n", "\n")
        .lines()
        .map(|line| line.strip_prefix("# ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");

    if editing {
        preview.push('_');
    }

    preview
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
