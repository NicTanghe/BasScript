const CANVAS_ZOOM_MIN: f32 = 0.1;
const CANVAS_ZOOM_MAX: f32 = 4.0;
const CANVAS_SCROLL_STEP_PX: f32 = 64.0;
const CANVAS_VIEW_MARGIN: f32 = 120.0;
const CANVAS_NODE_DEFAULT_WIDTH: f32 = 260.0;
const CANVAS_NODE_DEFAULT_HEIGHT: f32 = 160.0;

const COLOR_CANVAS_BG: Color = Color::srgb(0.38, 0.40, 0.43);

#[derive(Component)]
struct PanelCanvas {
    kind: PanelKind,
}

#[derive(Resource, Default)]
struct CanvasDragState {
    active: Option<CanvasDragMode>,
    last_cursor_position: Option<Vec2>,
}

enum CanvasDragMode {
    Pan { button: CanvasPanButton },
    MoveNode {
        node_id: String,
        undo_snapshot: Option<EditorHistorySnapshot>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CanvasPanButton {
    SpaceLeft,
    Middle,
}

impl EditorState {
    fn sync_canvas_document(&mut self) {
        if self.document_format != DocumentFormat::Canvas {
            self.canvas_document = None;
            self.canvas_parse_error = None;
            self.canvas_view_needs_centering = false;
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

    fn reset_canvas_view_to_content(&mut self) {
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

    fn center_canvas_view_in_panel(&mut self, panel_size: Vec2) {
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

    fn canvas_bounds(&self) -> Option<Rect> {
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

    fn canvas_world_from_panel_pos(&self, panel_pos: Vec2) -> Vec2 {
        self.canvas_pan + panel_pos / self.zoom.max(CANVAS_ZOOM_MIN)
    }

    fn canvas_node_index_at_world(&self, world: Vec2) -> Option<usize> {
        let canvas = self.canvas_document.as_ref()?;
        canvas.nodes.iter().enumerate().rev().find_map(|(index, node)| {
            let size = canvas_node_size(node.width, node.height);
            (world.x >= node.x
                && world.x <= node.x + size.x
                && world.y >= node.y
                && world.y <= node.y + size.y)
                .then_some(index)
        })
    }

    fn move_canvas_node_by_delta(&mut self, node_id: &str, delta: Vec2) -> bool {
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
}

fn canvas_node_size(width: f32, height: f32) -> Vec2 {
    Vec2::new(
        width.max(CANVAS_NODE_DEFAULT_WIDTH),
        height.max(CANVAS_NODE_DEFAULT_HEIGHT),
    )
}

fn handle_canvas_drag_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    panel_query: Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
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

    let cursor_position = window_query
        .iter()
        .next()
        .and_then(Window::cursor_position);
    let Some(cursor_position) = cursor_position else {
        return;
    };

    if drag_state.active.is_none() {
        start_canvas_drag_if_requested(
            &mouse_buttons,
            &keys,
            &panel_query,
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
    }
}

fn start_canvas_drag_if_requested(
    mouse_buttons: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
    panel_query: &Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
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

    if keys.pressed(KeyCode::Space) && mouse_buttons.just_pressed(MouseButton::Left) {
        start_canvas_pan_drag(drag_state, state, cursor_position, CanvasPanButton::SpaceLeft);
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Middle) {
        start_canvas_pan_drag(drag_state, state, cursor_position, CanvasPanButton::Middle);
    }
}

fn start_canvas_pan_drag(
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

fn canvas_drag_mode_still_active(
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
    }
}
