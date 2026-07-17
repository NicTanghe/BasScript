pub(crate) const CANVAS_ZOOM_MIN: f32 = 0.1;
pub(crate) const CANVAS_ZOOM_MAX: f32 = 4.0;
pub(crate) const CANVAS_SCROLL_STEP_PX: f32 = 64.0;
pub(crate) const CANVAS_VIEW_MARGIN: f32 = 120.0;
pub(crate) const CANVAS_NODE_DEFAULT_WIDTH: f32 = 260.0;
pub(crate) const CANVAS_NODE_DEFAULT_HEIGHT: f32 = 160.0;
pub(crate) const CANVAS_TEXT_PADDING_X: f32 = 10.0;
pub(crate) const CANVAS_TEXT_PADDING_Y: f32 = 10.0;

pub(crate) const COLOR_CANVAS_BG: Color = Color::srgb(0.38, 0.40, 0.43);

#[derive(Component)]
pub(crate) struct PanelCanvas {
    pub(crate) kind: PanelKind,
}

#[derive(Resource, Default)]
pub(crate) struct CanvasDragState {
    pub(crate) active: Option<CanvasDragMode>,
    pub(crate) last_cursor_position: Option<Vec2>,
}

pub(crate) enum CanvasDragMode {
    Pan {
        button: CanvasPanButton,
    },
    MoveNode {
        node_id: String,
        undo_snapshot: Option<EditorHistorySnapshot>,
    },
    SelectText {
        node_id: String,
        anchor: Position,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CanvasPanButton {
    SpaceLeft,
    Middle,
}

impl EditorState {
    pub(crate) fn sync_canvas_document(&mut self) {
        if self.document_format != DocumentFormat::Canvas {
            self.canvas_document = None;
            self.canvas_parse_error = None;
            self.canvas_view_needs_centering = false;
            self.canvas_editing_node_id = None;
            self.canvas_text_cursor = Cursor::default();
            self.canvas_text_selection_anchor = None;
            self.canvas_text_edit_undo_snapshot = None;
            self.canvas_text_suppress_next_insert_input = false;
            return;
        }

        match parse_canvas_document(&self.document.to_text()) {
            Ok(canvas) => {
                self.canvas_document = Some(canvas);
                self.canvas_parse_error = None;
            }
            Err(error) => {
                self.canvas_document = None;
                self.canvas_parse_error = Some(error.to_string());
            }
        }
        self.canvas_version = self.canvas_version.saturating_add(1);
    }

    pub(crate) fn reset_canvas_view_to_content(&mut self) {
        self.canvas_view_needs_centering = true;
        let Some(bounds) = self.canvas_bounds() else {
            self.canvas_pan = Vec2::ZERO;
            return;
        };

        self.canvas_pan = Vec2::new(
            bounds.min.x - CANVAS_VIEW_MARGIN,
            bounds.min.y - CANVAS_VIEW_MARGIN,
        );
    }

    pub(crate) fn center_canvas_view_in_panel(&mut self, panel_size: Vec2) {
        let Some(bounds) = self.canvas_bounds() else {
            self.canvas_pan = Vec2::ZERO;
            self.canvas_view_needs_centering = false;
            return;
        };

        let zoom = self.zoom.max(CANVAS_ZOOM_MIN);
        let content_center = (bounds.min + bounds.max) * 0.5;
        let viewport_half_size = panel_size / (zoom * 2.0);
        self.canvas_pan = content_center - viewport_half_size;
        self.canvas_view_needs_centering = false;
    }

    pub(crate) fn canvas_bounds(&self) -> Option<Rect> {
        let canvas = self.canvas_document.as_ref()?;
        let mut min = Vec2::new(f32::INFINITY, f32::INFINITY);
        let mut max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

        for node in &canvas.nodes {
            let size = canvas_node_size(node.width, node.height);
            min.x = min.x.min(node.x);
            min.y = min.y.min(node.y);
            max.x = max.x.max(node.x + size.x);
            max.y = max.y.max(node.y + size.y);
        }

        min.x.is_finite().then_some(Rect { min, max })
    }

    pub(crate) fn canvas_world_from_panel_pos(&self, panel_pos: Vec2) -> Vec2 {
        self.canvas_pan + panel_pos / self.zoom.max(CANVAS_ZOOM_MIN)
    }

    pub(crate) fn canvas_node_index_at_world(&self, world: Vec2) -> Option<usize> {
        let canvas = self.canvas_document.as_ref()?;
        canvas
            .nodes
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, node)| {
                let size = canvas_node_size(node.width, node.height);
                (world.x >= node.x
                    && world.x <= node.x + size.x
                    && world.y >= node.y
                    && world.y <= node.y + size.y)
                    .then_some(index)
            })
    }

    pub(crate) fn move_canvas_node_by_delta(&mut self, node_id: &str, delta: Vec2) -> bool {
        let Some((next_x, next_y)) = self
            .canvas_document
            .as_ref()
            .and_then(|canvas| canvas.nodes.iter().find(|node| node.id == node_id))
            .map(|node| (node.x + delta.x, node.y + delta.y))
        else {
            return false;
        };

        match update_canvas_node_position(&self.document.to_text(), node_id, next_x, next_y) {
            Ok(updated_document) => {
                self.document = Document::from_text(&updated_document);
                if let Some(node) = self
                    .canvas_document
                    .as_mut()
                    .and_then(|canvas| canvas.nodes.iter_mut().find(|node| node.id == node_id))
                {
                    node.x = next_x;
                    node.y = next_y;
                }
                self.canvas_parse_error = None;
                true
            }
            Err(error) => {
                self.canvas_parse_error = Some(error.to_string());
                self.status_message = format!("Canvas move failed: {error}");
                false
            }
        }
    }

    pub(crate) fn begin_canvas_text_edit(
        &mut self,
        node_id: String,
        cursor_position: Option<Position>,
    ) {
        self.workspace_focused = false;
        self.focused_panel = PanelKind::Processed;

        let changed_node = self.canvas_editing_node_id.as_deref() != Some(node_id.as_str());
        let previous_cursor = self.canvas_text_cursor.position;
        let previous_anchor = self.canvas_text_selection_anchor;
        if changed_node {
            if let Some(text) = self.canvas_text_node_content(&node_id) {
                self.canvas_text_cursor
                    .set_position(canvas_text_end_position(&text));
                self.canvas_text_selection_anchor = None;
            }
        }
        self.canvas_editing_node_id = Some(node_id);
        if let Some(cursor_position) = cursor_position {
            if let Some(document) = self.active_canvas_text_document() {
                self.canvas_text_cursor
                    .set_position(document.clamp_position(cursor_position));
                self.canvas_text_selection_anchor = None;
            }
        }
        if changed_node {
            self.canvas_text_edit_undo_snapshot = Some(self.history_snapshot());
        }
        if changed_node
            || previous_cursor != self.canvas_text_cursor.position
            || previous_anchor != self.canvas_text_selection_anchor
        {
            self.canvas_version = self.canvas_version.saturating_add(1);
        }
        self.status_message = if self.vim_enabled && self.vim_mode != VimMode::Insert {
            "Canvas text card selected. Vim normal mode: press i to edit.".to_string()
        } else {
            "Editing canvas text card. Esc exits.".to_string()
        };
        self.reset_blink();
    }

    pub(crate) fn clear_canvas_text_edit(&mut self) {
        let was_editing = self.canvas_editing_node_id.is_some();
        self.canvas_editing_node_id = None;
        self.canvas_text_cursor = Cursor::default();
        self.canvas_text_selection_anchor = None;
        self.canvas_text_edit_undo_snapshot = None;
        self.canvas_text_suppress_next_insert_input = false;
        if was_editing {
            self.canvas_version = self.canvas_version.saturating_add(1);
        }
        self.reset_blink();
    }

    pub(crate) fn canvas_text_node_content(&self, node_id: &str) -> Option<String> {
        self.canvas_document
            .as_ref()?
            .nodes
            .iter()
            .find(|node| node.id == node_id)
            .and_then(|node| match &node.kind {
                CanvasNodeKind::Text { text } => Some(text.clone()),
                _ => None,
            })
    }

    pub(crate) fn set_canvas_text_node_content(&mut self, node_id: &str, text: String) -> bool {
        match update_canvas_text_node_content(&self.document.to_text(), node_id, &text) {
            Ok(updated_document) => {
                self.document = Document::from_text(&updated_document);
                if let Some(node) = self
                    .canvas_document
                    .as_mut()
                    .and_then(|canvas| canvas.nodes.iter_mut().find(|node| node.id == node_id))
                {
                    node.kind = CanvasNodeKind::Text { text };
                }
                self.canvas_parse_error = None;
                self.canvas_version = self.canvas_version.saturating_add(1);
                true
            }
            Err(error) => {
                self.canvas_parse_error = Some(error.to_string());
                self.status_message = format!("Canvas text edit failed: {error}");
                false
            }
        }
    }

    pub(crate) fn active_canvas_text_document(&self) -> Option<Document> {
        let node_id = self.canvas_editing_node_id.as_deref()?;
        self.canvas_text_node_content(node_id)
            .map(|text| Document::from_text(&text))
    }

    pub(crate) fn active_canvas_text_node_index(&self) -> Option<usize> {
        let node_id = self.canvas_editing_node_id.as_deref()?;
        self.canvas_document
            .as_ref()?
            .nodes
            .iter()
            .position(|node| node.id == node_id)
    }

    pub(crate) fn canvas_text_selection_bounds(
        &self,
        document: &Document,
    ) -> Option<(Position, Position)> {
        let anchor = self.canvas_text_selection_anchor?;
        let anchor = document.clamp_position(anchor);
        let head = document.clamp_position(self.canvas_text_cursor.position);
        if anchor == head {
            return None;
        }

        if position_is_before_or_equal(anchor, head) {
            Some((anchor, head))
        } else {
            Some((head, anchor))
        }
    }

    pub(crate) fn set_canvas_text_cursor_with_selection(
        &mut self,
        document: &Document,
        position: Position,
        update_preferred: bool,
        extend_selection: bool,
    ) {
        self.canvas_text_selection_anchor = if extend_selection {
            Some(
                self.canvas_text_selection_anchor
                    .unwrap_or(self.canvas_text_cursor.position),
            )
        } else {
            None
        };

        let clamped = document.clamp_position(position);
        if update_preferred {
            self.canvas_text_cursor.set_position(clamped);
        } else {
            self.canvas_text_cursor.position = clamped;
        }
        self.reset_blink();
        self.canvas_version = self.canvas_version.saturating_add(1);
    }

    pub(crate) fn delete_canvas_text_selection(&mut self, document: &mut Document) -> bool {
        let Some((start, end)) = self.canvas_text_selection_bounds(document) else {
            return false;
        };
        let next = document.delete_range(start, end);
        self.canvas_text_cursor.set_position(next);
        self.canvas_text_selection_anchor = None;
        true
    }
}

pub(crate) fn canvas_text_end_position(text: &str) -> Position {
    let document = Document::from_text(text);
    let line = document.line_count().saturating_sub(1);
    Position {
        line,
        column: document.line_len_chars(line),
    }
}

pub(crate) fn canvas_text_position_from_world(
    node: &basscript_core::CanvasNode,
    text: &str,
    world_pos: Vec2,
    zoom: f32,
    layout: Option<CanvasTextLayout<'_>>,
) -> Position {
    let zoom = zoom.max(CANVAS_ZOOM_MIN);
    let local_screen = (world_pos - Vec2::new(node.x, node.y)) * zoom;
    let text_x = (local_screen.x - canvas_text_padding_x(zoom)).max(0.0);
    let text_y = (local_screen.y - canvas_text_padding_y(zoom)).max(0.0);
    let document = Document::from_text(text);
    let line_height = canvas_text_line_height(zoom);
    let char_width = canvas_text_char_width(zoom);
    let fallback_line =
        ((text_y / line_height).floor() as usize).min(document.line_count().saturating_sub(1));
    let fallback_column =
        ((text_x / char_width).round() as usize).min(document.line_len_chars(fallback_line));
    if let Some(layout) = layout {
        if let Some(position) = canvas_text_position_from_layout(&document, layout, text_x, text_y)
        {
            return position;
        }
    }

    Position {
        line: fallback_line,
        column: fallback_column,
    }
}

pub(crate) fn active_canvas_text_layout<'a>(
    state: &EditorState,
    text_layout_query: &'a Query<(
        &CanvasRenderedNodeText,
        &bevy::text::ComputedTextBlock,
        &ComputedNode,
    )>,
) -> Option<CanvasTextLayout<'a>> {
    let node_index = state.active_canvas_text_node_index()?;
    canvas_text_layout_for_node(text_layout_query, node_index)
}

pub(crate) fn move_canvas_text_cursor_by_key(
    state: &mut EditorState,
    document: &Document,
    key: KeyCode,
    extend_selection: bool,
    layout: Option<CanvasTextLayout<'_>>,
    zoom: f32,
) -> bool {
    let current = document.clamp_position(state.canvas_text_cursor.position);
    let next = match key {
        KeyCode::ArrowLeft => document.move_left(current),
        KeyCode::ArrowRight => document.move_right(current),
        KeyCode::ArrowUp => canvas_text_visual_arrow_position(document, current, key, layout, zoom)
            .unwrap_or_else(|| {
                document.move_up(current, state.canvas_text_cursor.preferred_column)
            }),
        KeyCode::ArrowDown => canvas_text_visual_arrow_position(
            document, current, key, layout, zoom,
        )
        .unwrap_or_else(|| document.move_down(current, state.canvas_text_cursor.preferred_column)),
        KeyCode::Home => Position {
            line: current.line,
            column: 0,
        },
        KeyCode::End => Position {
            line: current.line,
            column: document.line_len_chars(current.line),
        },
        _ => return false,
    };
    let update_preferred = !matches!(key, KeyCode::ArrowUp | KeyCode::ArrowDown);
    state.set_canvas_text_cursor_with_selection(document, next, update_preferred, extend_selection);
    next != current || extend_selection
}

pub(crate) fn canvas_text_visual_arrow_position(
    document: &Document,
    current: Position,
    key: KeyCode,
    layout: Option<CanvasTextLayout<'_>>,
    _zoom: f32,
) -> Option<Position> {
    let layout = layout.filter(|layout| canvas_text_layout_matches_document(*layout, document))?;
    let buffer = layout.block.buffer();
    let byte_index = canvas_text_document_byte_offset(document, current);
    let cursor = parley::Cursor::from_byte_index(buffer, byte_index, parley::Affinity::Downstream);
    let cursor_y = cursor.geometry(buffer, 0.0).y0 as f32;
    let visual_line = buffer
        .lines()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            let left_metrics = left.metrics();
            let right_metrics = right.metrics();
            let left_center = (left_metrics.block_min_coord + left_metrics.block_max_coord) * 0.5;
            let right_center =
                (right_metrics.block_min_coord + right_metrics.block_max_coord) * 0.5;
            (left_center - cursor_y)
                .abs()
                .partial_cmp(&(right_center - cursor_y).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(index, _)| index)?;
    match key {
        KeyCode::ArrowUp if visual_line == 0 => return None,
        KeyCode::ArrowDown if visual_line + 1 >= buffer.len() => return None,
        KeyCode::ArrowUp | KeyCode::ArrowDown => {}
        _ => return None,
    }

    let selection =
        parley::Selection::from_byte_index(buffer, cursor.index(), parley::Affinity::Downstream);
    let moved = match key {
        KeyCode::ArrowUp => selection.move_lines(buffer, -1, false),
        KeyCode::ArrowDown => selection.move_lines(buffer, 1, false),
        _ => return None,
    };
    Some(
        document.clamp_position(canvas_text_position_from_document_byte(
            document,
            moved.focus().index(),
        )),
    )
}

pub(crate) fn update_canvas_text_drag_selection(
    state: &mut EditorState,
    node_id: &str,
    anchor: Position,
    world_pos: Vec2,
    layout: Option<CanvasTextLayout<'_>>,
) -> bool {
    let Some((current, document)) = state.canvas_document.as_ref().and_then(|canvas| {
        canvas
            .nodes
            .iter()
            .find(|node| node.id == node_id)
            .and_then(|node| match &node.kind {
                CanvasNodeKind::Text { text } => Some((
                    canvas_text_position_from_world(node, text, world_pos, state.zoom, layout),
                    Document::from_text(text),
                )),
                _ => None,
            })
    }) else {
        return false;
    };

    let anchor = document.clamp_position(anchor);
    let current = document.clamp_position(current);
    let previous_cursor = state.canvas_text_cursor.position;
    let previous_anchor = state.canvas_text_selection_anchor;

    state.canvas_text_cursor.set_position(current);
    state.canvas_text_selection_anchor = (anchor != current).then_some(anchor);

    if state.vim_enabled && state.vim_mode != VimMode::Insert && anchor != current {
        state.vim_mode = VimMode::VisualChar;
        state.vim_pending_operator = None;
        state.vim_visual_anchor = Some(anchor);
        state.vim_visual_head = Some(current);
        state.status_message = "Vim visual mode.".to_string();
    }

    if previous_cursor != state.canvas_text_cursor.position
        || previous_anchor != state.canvas_text_selection_anchor
    {
        state.canvas_version = state.canvas_version.saturating_add(1);
        state.reset_blink();
        return true;
    }

    false
}

pub(crate) fn canvas_text_editable_key(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::ArrowLeft
            | KeyCode::ArrowRight
            | KeyCode::ArrowUp
            | KeyCode::ArrowDown
            | KeyCode::Home
            | KeyCode::End
    )
}

pub(crate) fn canvas_text_arrow_key(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::ArrowUp | KeyCode::ArrowDown
    )
}

pub(crate) fn canvas_text_font_size(zoom: f32) -> f32 {
    FONT_SIZE * zoom.max(CANVAS_ZOOM_MIN)
}

pub(crate) fn canvas_text_line_height(zoom: f32) -> f32 {
    canvas_text_font_size(zoom) * 1.25
}

pub(crate) fn canvas_text_char_width(zoom: f32) -> f32 {
    canvas_text_font_size(zoom) * 0.55
}

pub(crate) fn canvas_text_padding_x(zoom: f32) -> f32 {
    CANVAS_TEXT_PADDING_X * zoom.max(CANVAS_ZOOM_MIN)
}

pub(crate) fn canvas_text_padding_y(zoom: f32) -> f32 {
    CANVAS_TEXT_PADDING_Y * zoom.max(CANVAS_ZOOM_MIN)
}

pub(crate) fn canvas_node_size(width: f32, height: f32) -> Vec2 {
    Vec2::new(
        width.max(CANVAS_NODE_DEFAULT_WIDTH),
        height.max(CANVAS_NODE_DEFAULT_HEIGHT),
    )
}

pub(crate) fn handle_canvas_drag_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    panel_query: Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
    text_layout_query: Query<(
        &CanvasRenderedNodeText,
        &bevy::text::ComputedTextBlock,
        &ComputedNode,
    )>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut drag_state: ResMut<CanvasDragState>,
    mut state: ResMut<EditorState>,
) {
    if state.document_format != DocumentFormat::Canvas {
        drag_state.active = None;
        drag_state.last_cursor_position = None;
        return;
    }

    if state.workspace_prompt.is_some() || state.command_menu.is_some() {
        drag_state.active = None;
        drag_state.last_cursor_position = None;
        return;
    }

    let cursor_position = window_query.iter().next().and_then(Window::cursor_position);
    let Some(cursor_position) = cursor_position else {
        return;
    };

    if drag_state.active.is_none() {
        start_canvas_drag_if_requested(
            &mouse_buttons,
            &keys,
            &panel_query,
            &text_layout_query,
            &mut drag_state,
            &mut state,
            cursor_position,
        );
        return;
    }

    let Some(previous_cursor_position) = drag_state.last_cursor_position else {
        drag_state.last_cursor_position = Some(cursor_position);
        return;
    };
    let delta = cursor_position - previous_cursor_position;
    drag_state.last_cursor_position = Some(cursor_position);

    let Some(mode) = drag_state.active.as_mut() else {
        return;
    };
    if !canvas_drag_mode_still_active(mode, &mouse_buttons, &keys) {
        drag_state.active = None;
        drag_state.last_cursor_position = None;
        return;
    }
    if delta.length_squared() <= f32::EPSILON {
        return;
    }

    let zoom = state.zoom.max(CANVAS_ZOOM_MIN);
    match mode {
        CanvasDragMode::Pan { .. } => {
            state.canvas_pan.x -= delta.x / zoom;
            state.canvas_pan.y -= delta.y / zoom;
        }
        CanvasDragMode::MoveNode {
            node_id,
            undo_snapshot,
        } => {
            if let Some(snapshot) = undo_snapshot.take() {
                state.push_undo_snapshot(snapshot);
            }
            state.move_canvas_node_by_delta(node_id, delta / zoom);
        }
        CanvasDragMode::SelectText { node_id, anchor } => {
            let panel_context = gather_scroll_panels_context(&panel_query, &state);
            let Some(panel_pos) = panel_context.processed_cursor_pos else {
                return;
            };
            let world_pos = state.canvas_world_from_panel_pos(panel_pos);
            let layout = state
                .canvas_document
                .as_ref()
                .and_then(|canvas| {
                    canvas
                        .nodes
                        .iter()
                        .position(|node| node.id.as_str() == node_id.as_str())
                })
                .and_then(|node_index| canvas_text_layout_for_node(&text_layout_query, node_index));
            update_canvas_text_drag_selection(&mut state, node_id, *anchor, world_pos, layout);
        }
    }
}

pub(crate) fn start_canvas_drag_if_requested(
    mouse_buttons: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
    panel_query: &Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
    text_layout_query: &Query<(
        &CanvasRenderedNodeText,
        &bevy::text::ComputedTextBlock,
        &ComputedNode,
    )>,
    drag_state: &mut CanvasDragState,
    state: &mut EditorState,
    cursor_position: Vec2,
) {
    let panel_context = gather_scroll_panels_context(panel_query, state);
    if panel_context.hovered_panel != Some(PanelKind::Processed) {
        return;
    }

    let ctrl_pressed = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if ctrl_pressed && mouse_buttons.just_pressed(MouseButton::Left) {
        let Some(panel_pos) = panel_context.processed_cursor_pos else {
            return;
        };
        let world_pos = state.canvas_world_from_panel_pos(panel_pos);
        let Some(node_index) = state.canvas_node_index_at_world(world_pos) else {
            return;
        };
        let Some(node_id) = state
            .canvas_document
            .as_ref()
            .and_then(|canvas| canvas.nodes.get(node_index))
            .map(|node| node.id.clone())
        else {
            return;
        };

        state.workspace_focused = false;
        state.focused_panel = PanelKind::Processed;
        drag_state.active = Some(CanvasDragMode::MoveNode {
            node_id,
            undo_snapshot: Some(state.history_snapshot()),
        });
        drag_state.last_cursor_position = Some(cursor_position);
        return;
    }

    if !ctrl_pressed
        && !keys.pressed(KeyCode::Space)
        && mouse_buttons.just_pressed(MouseButton::Left)
    {
        let Some(panel_pos) = panel_context.processed_cursor_pos else {
            state.clear_canvas_text_edit();
            return;
        };
        let world_pos = state.canvas_world_from_panel_pos(panel_pos);
        let Some(node_index) = state.canvas_node_index_at_world(world_pos) else {
            state.clear_canvas_text_edit();
            return;
        };
        let Some((node_id, click_position)) = state
            .canvas_document
            .as_ref()
            .and_then(|canvas| canvas.nodes.get(node_index))
            .map(|node| match &node.kind {
                CanvasNodeKind::Text { text } => (
                    node.id.clone(),
                    Some(canvas_text_position_from_world(
                        node,
                        text,
                        world_pos,
                        state.zoom,
                        canvas_text_layout_for_node(text_layout_query, node_index),
                    )),
                ),
                _ => (node.id.clone(), None),
            })
        else {
            return;
        };
        if let Some(click_position) = click_position {
            state.begin_canvas_text_edit(node_id.clone(), Some(click_position));
            drag_state.active = Some(CanvasDragMode::SelectText {
                node_id,
                anchor: click_position,
            });
            drag_state.last_cursor_position = Some(cursor_position);
        } else {
            state.clear_canvas_text_edit();
        }
        return;
    }

    if keys.pressed(KeyCode::Space) && mouse_buttons.just_pressed(MouseButton::Left) {
        start_canvas_pan_drag(
            drag_state,
            state,
            cursor_position,
            CanvasPanButton::SpaceLeft,
        );
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Middle) {
        start_canvas_pan_drag(drag_state, state, cursor_position, CanvasPanButton::Middle);
    }
}

pub(crate) fn start_canvas_pan_drag(
    drag_state: &mut CanvasDragState,
    state: &mut EditorState,
    cursor_position: Vec2,
    button: CanvasPanButton,
) {
    state.workspace_focused = false;
    state.focused_panel = PanelKind::Processed;
    drag_state.active = Some(CanvasDragMode::Pan { button });
    drag_state.last_cursor_position = Some(cursor_position);
}

pub(crate) fn canvas_drag_mode_still_active(
    mode: &CanvasDragMode,
    mouse_buttons: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
) -> bool {
    match mode {
        CanvasDragMode::Pan {
            button: CanvasPanButton::SpaceLeft,
        } => keys.pressed(KeyCode::Space) && mouse_buttons.pressed(MouseButton::Left),
        CanvasDragMode::Pan {
            button: CanvasPanButton::Middle,
        } => mouse_buttons.pressed(MouseButton::Middle),
        CanvasDragMode::MoveNode { .. } => mouse_buttons.pressed(MouseButton::Left),
        CanvasDragMode::SelectText { .. } => mouse_buttons.pressed(MouseButton::Left),
    }
}

pub(crate) fn handle_canvas_text_edit_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    capture: Res<LinkAutocompleteInputCapture>,
    text_layout_query: Query<(
        &CanvasRenderedNodeText,
        &bevy::text::ComputedTextBlock,
        &ComputedNode,
    )>,
    mut navigation_repeat: ResMut<NavigationRepeatState>,
    mut state: ResMut<EditorState>,
) {
    if state.document_format != DocumentFormat::Canvas {
        return;
    }

    if capture.is_captured() {
        for _ in keyboard_inputs.read() {}
        return;
    }

    if state.workspace_prompt.is_some()
        || state.command_menu.is_some()
        || state.story_query_sheet.open
    {
        state.close_link_autocomplete();
        return;
    }

    let Some(node_id) = state.canvas_editing_node_id.clone() else {
        state.close_link_autocomplete();
        return;
    };

    if state.vim_enabled {
        if state.canvas_text_suppress_next_insert_input {
            for _ in keyboard_inputs.read() {}
            state.canvas_text_suppress_next_insert_input = false;
            return;
        }

        if keys.just_pressed(KeyCode::Escape) || state.vim_mode != VimMode::Insert {
            state.close_link_autocomplete();
            return;
        }
    } else if keys.just_pressed(KeyCode::Escape) {
        state.clear_canvas_text_edit();
        state.close_link_autocomplete();
        state.status_message = "Canvas text edit exited.".to_string();
        return;
    }

    let Some(mut document) = state.active_canvas_text_document() else {
        state.clear_canvas_text_edit();
        state.close_link_autocomplete();
        return;
    };

    if copy_shortcut_just_pressed(&keys) {
        copy_canvas_text_selection_to_clipboard(&mut state, &document);
        for _ in keyboard_inputs.read() {}
        return;
    }

    if cut_shortcut_just_pressed(&keys) {
        if cut_canvas_text_selection_to_clipboard(&mut state, &mut document) {
            if let Some(snapshot) = state.canvas_text_edit_undo_snapshot.take() {
                state.push_undo_snapshot(snapshot);
            }
            if state.set_canvas_text_node_content(&node_id, document.to_text()) {
                state.refresh_link_autocomplete_for_canvas_text_cursor(&document);
                state.status_message = "Cut canvas text.".to_string();
            }
        }
        for _ in keyboard_inputs.read() {}
        return;
    }

    if paste_shortcut_just_pressed(&keys) {
        if let Some(text) = read_system_clipboard_text() {
            state.delete_canvas_text_selection(&mut document);
            let next = document.insert_text(state.canvas_text_cursor.position, &text);
            state.canvas_text_cursor.set_position(next);
            state.canvas_text_selection_anchor = None;
            state.vim_register = Some(VimRegister::Characterwise(text));
            if let Some(snapshot) = state.canvas_text_edit_undo_snapshot.take() {
                state.push_undo_snapshot(snapshot);
            }
            if state.set_canvas_text_node_content(&node_id, document.to_text()) {
                state.refresh_link_autocomplete_for_canvas_text_cursor(&document);
                state.status_message = "Pasted clipboard.".to_string();
            }
        } else {
            state.status_message = "Clipboard is empty or unavailable.".to_string();
        }
        for _ in keyboard_inputs.read() {}
        return;
    }

    if platform_shortcut_modifier_pressed(&keys) && keys.just_pressed(KeyCode::KeyA) {
        let end = canvas_text_end_position(&document.to_text());
        state.canvas_text_cursor.set_position(end);
        state.canvas_text_selection_anchor = Some(Position::default());
        state.canvas_version = state.canvas_version.saturating_add(1);
        state.close_link_autocomplete();
        state.status_message = "Canvas text selected.".to_string();
        return;
    }

    let mut text_changed = false;
    let mut view_changed = false;
    let active_layout = active_canvas_text_layout(&state, &text_layout_query);
    let zoom = state.zoom;
    let extend_selection = shift_modifier_pressed(&keys);

    view_changed |= repeat_navigation_arrow_input(&keys, &time, &mut navigation_repeat, |arrow| {
        move_canvas_text_cursor_by_key(
            &mut state,
            &document,
            arrow,
            extend_selection,
            active_layout,
            zoom,
        )
    });

    for key_input in keyboard_inputs.read() {
        if !key_input.state.is_pressed() {
            continue;
        }

        if text_input_should_skip_for_shortcut(&keys, key_input, &state.keybinds) {
            continue;
        }

        if canvas_text_arrow_key(key_input.key_code) {
            continue;
        }

        if canvas_text_editable_key(key_input.key_code) {
            view_changed |= move_canvas_text_cursor_by_key(
                &mut state,
                &document,
                key_input.key_code,
                extend_selection,
                active_layout,
                zoom,
            );
            continue;
        }

        match &key_input.logical_key {
            Key::Enter => {
                state.delete_canvas_text_selection(&mut document);
                let next = document.insert_newline(state.canvas_text_cursor.position);
                state.canvas_text_cursor.set_position(next);
                state.canvas_text_selection_anchor = None;
                text_changed = true;
            }
            Key::Backspace => {
                if !state.delete_canvas_text_selection(&mut document) {
                    let previous = state.canvas_text_cursor.position;
                    let next = document.backspace(previous);
                    text_changed |= next != previous;
                    state.canvas_text_cursor.set_position(next);
                } else {
                    text_changed = true;
                }
            }
            Key::Delete => {
                if !state.delete_canvas_text_selection(&mut document) {
                    let previous = state.canvas_text_cursor.position;
                    let next = document.delete(previous);
                    text_changed |= next != previous;
                    state.canvas_text_cursor.set_position(next);
                } else {
                    text_changed = true;
                }
            }
            _ => {
                if let Some(inserted_text) = &key_input.text {
                    if !inserted_text.is_empty() && inserted_text.chars().all(is_printable_char) {
                        state.delete_canvas_text_selection(&mut document);
                        let next =
                            document.insert_text(state.canvas_text_cursor.position, inserted_text);
                        state.canvas_text_cursor.set_position(next);
                        state.canvas_text_selection_anchor = None;
                        text_changed = true;
                    }
                }
            }
        }
    }

    if !text_changed {
        if view_changed {
            state.validate_link_autocomplete_context();
            state.reset_blink();
        }
        return;
    }

    if let Some(snapshot) = state.canvas_text_edit_undo_snapshot.take() {
        state.push_undo_snapshot(snapshot);
    }
    state.set_canvas_text_node_content(&node_id, document.to_text());
    state.refresh_link_autocomplete_for_canvas_text_cursor(&document);
    state.reset_blink();
}
#[allow(unused_imports)]
use super::*;
