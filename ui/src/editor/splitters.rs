fn sync_panel_split_layout(
    state: Res<EditorState>,
    mut layout: ResMut<PanelLayoutState>,
    body_row_query: Query<&ComputedNode, With<EditorBodyRow>>,
    mut node_queries: ParamSet<(
        Query<&mut Node, With<WorkspaceSidebarPane>>,
        Query<&mut Node, With<EditorPanelsContainer>>,
        Query<(&PanelPaneSlot, &mut Node)>,
        Query<(&PanelSplitter, &mut Node)>,
    )>,
) {
    let Some(body_row) = body_row_query.iter().next() else {
        return;
    };
    let total_width = body_row.size().x * body_row.inverse_scale_factor();
    if total_width <= 0.0 {
        return;
    }

    let workspace_width = clamp_workspace_width(&mut layout, total_width, state.display_mode);
    let editor_width = (total_width - PANEL_SPLITTER_WIDTH - workspace_width).max(0.0);

    let split_available = (editor_width - PANEL_SPLITTER_WIDTH).max(0.0);
    let split_is_visible = state.display_mode == DisplayMode::Split;
    let plain_width = if split_is_visible {
        clamp_plain_width_from_ratio(&mut layout, split_available)
    } else {
        0.0
    };
    let processed_width = if split_is_visible {
        (split_available - plain_width).max(0.0)
    } else {
        0.0
    };

    for mut node in node_queries.p0().iter_mut() {
        node.width = px(workspace_width);
        node.display = Display::Flex;
    }

    for mut node in node_queries.p1().iter_mut() {
        node.width = px(editor_width);
    }

    for (pane_slot, mut node) in node_queries.p2().iter_mut() {
        match (state.display_mode, pane_slot.kind) {
            (DisplayMode::Split, PanelKind::Plain) => {
                node.display = Display::Flex;
                node.width = px(plain_width);
            }
            (DisplayMode::Split, PanelKind::Processed) => {
                node.display = Display::Flex;
                node.width = px(processed_width);
            }
            (DisplayMode::Plain, PanelKind::Plain) => {
                node.display = Display::Flex;
                node.width = px(editor_width);
            }
            (DisplayMode::Processed, PanelKind::Processed)
            | (DisplayMode::ProcessedRawCurrentLine, PanelKind::Processed) => {
                node.display = Display::Flex;
                node.width = px(editor_width);
            }
            _ => {
                node.display = Display::None;
                node.width = px(0.0);
            }
        }
    }

    for (splitter, mut node) in node_queries.p3().iter_mut() {
        node.width = px(PANEL_SPLITTER_WIDTH);
        node.display = if splitter_visible_for_mode(*splitter, state.display_mode) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn handle_panel_splitter_drag(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    body_row_query: Query<(&ComputedNode, &RelativeCursorPosition), With<EditorBodyRow>>,
    state: Res<EditorState>,
    mut layout: ResMut<PanelLayoutState>,
    mut drag_state: ResMut<PanelSplitterDragState>,
) {
    if drag_state.suppress_next_left_click && !mouse_buttons.pressed(MouseButton::Left) {
        drag_state.suppress_next_left_click = false;
    }

    if mouse_buttons.just_pressed(MouseButton::Left)
        && !keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
    {
        let hovered_splitter =
            body_row_query
                .iter()
                .next()
                .and_then(|(computed, relative_cursor)| {
                    if !relative_cursor.cursor_over() {
                        return None;
                    }
                    let normalized = relative_cursor.normalized?;
                    let total_width = computed.size().x * computed.inverse_scale_factor();
                    if total_width <= 0.0 {
                        return None;
                    }
                    let local_x = (normalized.x + 0.5) * total_width;
                    splitter_from_cursor_x(local_x, total_width, state.display_mode, &mut layout)
                });

        if let Some(splitter) = hovered_splitter {
            drag_state.active = Some(splitter);
            drag_state.last_cursor_x = primary_cursor_x(&window_query);
            drag_state.suppress_next_left_click = true;
        }
    }

    if !mouse_buttons.pressed(MouseButton::Left) {
        drag_state.active = None;
        drag_state.last_cursor_x = None;
        return;
    }

    let Some(active_splitter) = drag_state.active else {
        return;
    };
    let Some(cursor_x) = primary_cursor_x(&window_query) else {
        return;
    };
    let previous_x = drag_state.last_cursor_x.unwrap_or(cursor_x);
    let delta_x = cursor_x - previous_x;
    drag_state.last_cursor_x = Some(cursor_x);

    if delta_x.abs() < f32::EPSILON {
        return;
    }

    let Some((body_row, _)) = body_row_query.iter().next() else {
        return;
    };
    let total_width = body_row.size().x * body_row.inverse_scale_factor();
    if total_width <= 0.0 {
        return;
    }

    match active_splitter {
        PanelSplitter::Workspace => {
            let workspace_width =
                clamp_workspace_width(&mut layout, total_width, state.display_mode) + delta_x;
            let min_editor_width = min_editor_content_width(state.display_mode);
            let max_workspace_width = (total_width - PANEL_SPLITTER_WIDTH - min_editor_width).max(0.0);
            let min_workspace_width = WORKSPACE_WIDTH_MIN.min(max_workspace_width);
            layout.workspace_width_px = workspace_width.clamp(min_workspace_width, max_workspace_width);
        }
        PanelSplitter::Panels => {
            if state.display_mode != DisplayMode::Split {
                return;
            }
            let workspace_width = clamp_workspace_width(&mut layout, total_width, state.display_mode);
            let editor_width = (total_width - PANEL_SPLITTER_WIDTH - workspace_width).max(0.0);
            let split_available = (editor_width - PANEL_SPLITTER_WIDTH).max(0.0);
            if split_available <= 0.0 {
                layout.plain_ratio = 0.5;
                return;
            }

            let current_plain = layout.plain_ratio * split_available;
            let next_plain = if split_available > EDITOR_PANEL_MIN_WIDTH * 2.0 {
                (current_plain + delta_x)
                    .clamp(EDITOR_PANEL_MIN_WIDTH, split_available - EDITOR_PANEL_MIN_WIDTH)
            } else {
                split_available * 0.5
            };
            layout.plain_ratio = (next_plain / split_available).clamp(0.0, 1.0);
        }
    }
}

fn style_panel_splitters(
    state: Res<EditorState>,
    drag_state: Res<PanelSplitterDragState>,
    mut splitter_query: Query<(&PanelSplitter, &RelativeCursorPosition, &mut BackgroundColor)>,
) {
    for (splitter, relative_cursor, mut color) in splitter_query.iter_mut() {
        color.0 = if !splitter_visible_for_mode(*splitter, state.display_mode) {
            Color::srgba(0.0, 0.0, 0.0, 0.0)
        } else if drag_state.active == Some(*splitter) {
            COLOR_SPLITTER_ACTIVE
        } else if relative_cursor.cursor_over() {
            COLOR_SPLITTER_HOVER
        } else {
            COLOR_SPLITTER_IDLE
        };
    }
}

fn primary_cursor_x(window_query: &Query<&Window, With<PrimaryWindow>>) -> Option<f32> {
    window_query
        .iter()
        .next()
        .and_then(Window::cursor_position)
        .map(|position| position.x)
}

fn splitter_visible_for_mode(splitter: PanelSplitter, display_mode: DisplayMode) -> bool {
    match splitter {
        PanelSplitter::Workspace => true,
        PanelSplitter::Panels => display_mode == DisplayMode::Split,
    }
}

fn min_editor_content_width(display_mode: DisplayMode) -> f32 {
    if display_mode == DisplayMode::Split {
        EDITOR_PANEL_MIN_WIDTH * 2.0 + PANEL_SPLITTER_WIDTH
    } else {
        EDITOR_PANEL_MIN_WIDTH
    }
}

fn clamp_workspace_width(
    layout: &mut PanelLayoutState,
    total_width: f32,
    display_mode: DisplayMode,
) -> f32 {
    let max_workspace_width =
        (total_width - PANEL_SPLITTER_WIDTH - min_editor_content_width(display_mode)).max(0.0);
    let min_workspace_width = WORKSPACE_WIDTH_MIN.min(max_workspace_width);
    layout.workspace_width_px = layout
        .workspace_width_px
        .clamp(min_workspace_width, max_workspace_width);
    layout.workspace_width_px
}

fn clamp_plain_width_from_ratio(layout: &mut PanelLayoutState, split_available: f32) -> f32 {
    if split_available <= 0.0 {
        layout.plain_ratio = 0.5;
        return 0.0;
    }

    let width = if split_available > EDITOR_PANEL_MIN_WIDTH * 2.0 {
        (layout.plain_ratio * split_available)
            .clamp(EDITOR_PANEL_MIN_WIDTH, split_available - EDITOR_PANEL_MIN_WIDTH)
    } else {
        split_available * 0.5
    };
    layout.plain_ratio = (width / split_available).clamp(0.0, 1.0);
    width
}

fn splitter_from_cursor_x(
    local_x: f32,
    total_width: f32,
    display_mode: DisplayMode,
    layout: &mut PanelLayoutState,
) -> Option<PanelSplitter> {
    let workspace_width = clamp_workspace_width(layout, total_width, display_mode);
    let workspace_center = workspace_width + PANEL_SPLITTER_WIDTH * 0.5;
    let mut closest = (local_x - workspace_center).abs();
    let mut result = PanelSplitter::Workspace;

    if display_mode == DisplayMode::Split {
        let editor_width = (total_width - PANEL_SPLITTER_WIDTH - workspace_width).max(0.0);
        let split_available = (editor_width - PANEL_SPLITTER_WIDTH).max(0.0);
        let plain_width = clamp_plain_width_from_ratio(layout, split_available);
        let panels_center =
            workspace_width + PANEL_SPLITTER_WIDTH + plain_width + PANEL_SPLITTER_WIDTH * 0.5;
        let panel_distance = (local_x - panels_center).abs();
        if panel_distance < closest {
            closest = panel_distance;
            result = PanelSplitter::Panels;
        }
    }

    if closest <= PANEL_SPLITTER_PICK_RADIUS {
        Some(result)
    } else {
        None
    }
}
