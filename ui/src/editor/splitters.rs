pub(crate) fn sync_panel_split_layout(
    state: Res<EditorState>,
    mut layout: ResMut<PanelLayoutState>,
    mut resolved_widths: ResMut<ResolvedPanelWidths>,
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
    let layout_display_mode = state.panel_layout_display_mode();

    let workspace_width = effective_workspace_width(
        &mut layout,
        total_width,
        layout_display_mode,
        state.workspace_sidebar_visible,
    );
    let workspace_splitter_width = if state.workspace_sidebar_visible {
        PANEL_SPLITTER_WIDTH
    } else {
        0.0
    };
    let editor_width = (total_width - workspace_splitter_width - workspace_width).max(0.0);

    let split_available = (editor_width - PANEL_SPLITTER_WIDTH).max(0.0);
    let split_is_visible = layout_display_mode == DisplayMode::Split;
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

    let (resolved_plain_width, resolved_processed_width) = target_panel_widths(
        layout_display_mode,
        state.document_format,
        editor_width,
        plain_width,
        processed_width,
    );
    resolved_widths.set(resolved_plain_width, resolved_processed_width);

    for mut node in node_queries.p0().iter_mut() {
        node.width = px(workspace_width);
        node.display = if state.workspace_sidebar_visible {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in node_queries.p1().iter_mut() {
        node.width = px(editor_width);
    }

    for (pane_slot, mut node) in node_queries.p2().iter_mut() {
        match (layout_display_mode, pane_slot.kind) {
            _ if state.document_format == DocumentFormat::Canvas
                && pane_slot.kind == PanelKind::Processed =>
            {
                node.display = Display::Flex;
                node.width = px(editor_width);
            }
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
        node.display = if splitter_visible_for_mode(
            *splitter,
            layout_display_mode,
            state.workspace_sidebar_visible,
        ) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub(crate) fn handle_panel_splitter_drag(
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
        let workspace_sidebar_visible = state.workspace_sidebar_visible;
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
                    splitter_from_cursor_x(
                        local_x,
                        total_width,
                        state.panel_layout_display_mode(),
                        workspace_sidebar_visible,
                        &mut layout,
                    )
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
            if !state.workspace_sidebar_visible {
                return;
            }
            let workspace_width = effective_workspace_width(
                &mut layout,
                total_width,
                state.panel_layout_display_mode(),
                state.workspace_sidebar_visible,
            ) + delta_x;
            let min_editor_width = min_editor_content_width(state.panel_layout_display_mode());
            let max_workspace_width =
                (total_width - PANEL_SPLITTER_WIDTH - min_editor_width).max(0.0);
            let min_workspace_width = WORKSPACE_WIDTH_MIN.min(max_workspace_width);
            layout.workspace_width_px =
                workspace_width.clamp(min_workspace_width, max_workspace_width);
        }
        PanelSplitter::Panels => {
            if state.panel_layout_display_mode() != DisplayMode::Split {
                return;
            }
            let workspace_width = effective_workspace_width(
                &mut layout,
                total_width,
                state.panel_layout_display_mode(),
                state.workspace_sidebar_visible,
            );
            let workspace_splitter_width = if state.workspace_sidebar_visible {
                PANEL_SPLITTER_WIDTH
            } else {
                0.0
            };
            let editor_width = (total_width - workspace_splitter_width - workspace_width).max(0.0);
            let split_available = (editor_width - PANEL_SPLITTER_WIDTH).max(0.0);
            if split_available <= 0.0 {
                layout.plain_ratio = 0.5;
                return;
            }

            let current_plain = layout.plain_ratio * split_available;
            let next_plain = if split_available > EDITOR_PANEL_MIN_WIDTH * 2.0 {
                (current_plain + delta_x).clamp(
                    EDITOR_PANEL_MIN_WIDTH,
                    split_available - EDITOR_PANEL_MIN_WIDTH,
                )
            } else {
                split_available * 0.5
            };
            layout.plain_ratio = (next_plain / split_available).clamp(0.0, 1.0);
        }
    }
}

pub(crate) fn style_panel_splitters(
    state: Res<EditorState>,
    drag_state: Res<PanelSplitterDragState>,
    mut splitter_query: Query<(
        &PanelSplitter,
        &RelativeCursorPosition,
        &mut BackgroundColor,
    )>,
) {
    for (splitter, relative_cursor, mut color) in splitter_query.iter_mut() {
        color.0 = if !splitter_visible_for_mode(
            *splitter,
            state.panel_layout_display_mode(),
            state.workspace_sidebar_visible,
        ) {
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

pub(crate) fn primary_cursor_x(window_query: &Query<&Window, With<PrimaryWindow>>) -> Option<f32> {
    window_query
        .iter()
        .next()
        .and_then(Window::cursor_position)
        .map(|position| position.x)
}

pub(crate) fn splitter_visible_for_mode(
    splitter: PanelSplitter,
    display_mode: DisplayMode,
    workspace_sidebar_visible: bool,
) -> bool {
    match splitter {
        PanelSplitter::Workspace => workspace_sidebar_visible,
        PanelSplitter::Panels => display_mode == DisplayMode::Split,
    }
}

pub(crate) fn min_editor_content_width(display_mode: DisplayMode) -> f32 {
    if display_mode == DisplayMode::Split {
        EDITOR_PANEL_MIN_WIDTH * 2.0 + PANEL_SPLITTER_WIDTH
    } else {
        EDITOR_PANEL_MIN_WIDTH
    }
}

pub(crate) fn clamp_workspace_width(
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

pub(crate) fn clamp_plain_width_from_ratio(
    layout: &mut PanelLayoutState,
    split_available: f32,
) -> f32 {
    if split_available <= 0.0 {
        layout.plain_ratio = 0.5;
        return 0.0;
    }

    let width = if split_available > EDITOR_PANEL_MIN_WIDTH * 2.0 {
        (layout.plain_ratio * split_available).clamp(
            EDITOR_PANEL_MIN_WIDTH,
            split_available - EDITOR_PANEL_MIN_WIDTH,
        )
    } else {
        split_available * 0.5
    };
    layout.plain_ratio = (width / split_available).clamp(0.0, 1.0);
    width
}

pub(crate) fn splitter_from_cursor_x(
    local_x: f32,
    total_width: f32,
    display_mode: DisplayMode,
    workspace_sidebar_visible: bool,
    layout: &mut PanelLayoutState,
) -> Option<PanelSplitter> {
    let workspace_width =
        effective_workspace_width(layout, total_width, display_mode, workspace_sidebar_visible);
    let mut closest = f32::INFINITY;
    let mut result = PanelSplitter::Panels;

    if workspace_sidebar_visible {
        let workspace_center = workspace_width + PANEL_SPLITTER_WIDTH * 0.5;
        closest = (local_x - workspace_center).abs();
        result = PanelSplitter::Workspace;
    }

    if display_mode == DisplayMode::Split {
        let workspace_splitter_width = if workspace_sidebar_visible {
            PANEL_SPLITTER_WIDTH
        } else {
            0.0
        };
        let editor_width = (total_width - workspace_splitter_width - workspace_width).max(0.0);
        let split_available = (editor_width - PANEL_SPLITTER_WIDTH).max(0.0);
        let plain_width = clamp_plain_width_from_ratio(layout, split_available);
        let panels_center =
            workspace_width + workspace_splitter_width + plain_width + PANEL_SPLITTER_WIDTH * 0.5;
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

pub(crate) fn effective_workspace_width(
    layout: &mut PanelLayoutState,
    total_width: f32,
    display_mode: DisplayMode,
    workspace_sidebar_visible: bool,
) -> f32 {
    if workspace_sidebar_visible {
        clamp_workspace_width(layout, total_width, display_mode)
    } else {
        0.0
    }
}

fn target_panel_widths(
    display_mode: DisplayMode,
    document_format: DocumentFormat,
    editor_width: f32,
    plain_width: f32,
    processed_width: f32,
) -> (f32, f32) {
    if document_format == DocumentFormat::Canvas {
        return (0.0, editor_width);
    }

    match display_mode {
        DisplayMode::Split => (plain_width, processed_width),
        DisplayMode::Plain => (editor_width, 0.0),
        DisplayMode::Processed | DisplayMode::ProcessedRawCurrentLine => (0.0, editor_width),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_panel_width_uses_current_explorer_adjusted_editor_width() {
        let total_width = 1_600.0;
        let explorer_width = 300.0;
        let (_, closed_width) = target_panel_widths(
            DisplayMode::Processed,
            DocumentFormat::Markdown,
            total_width,
            0.0,
            0.0,
        );
        let (_, open_width) = target_panel_widths(
            DisplayMode::Processed,
            DocumentFormat::Markdown,
            total_width - explorer_width,
            0.0,
            0.0,
        );

        assert_eq!(closed_width, 1_600.0);
        assert_eq!(open_width, 1_300.0);
    }

    #[test]
    fn canvas_always_uses_the_processed_panel_target_width() {
        let widths = target_panel_widths(
            DisplayMode::Plain,
            DocumentFormat::Canvas,
            900.0,
            450.0,
            450.0,
        );

        assert_eq!(widths, (0.0, 900.0));
    }
}
#[allow(unused_imports)]
use super::*;
