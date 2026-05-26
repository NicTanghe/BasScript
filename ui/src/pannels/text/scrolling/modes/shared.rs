#[derive(Clone, Copy)]
struct ScrollPanelsContext {
    plain_panel_size: Option<Vec2>,
    processed_panel_size: Option<Vec2>,
    hovered_panel: Option<PanelKind>,
    processed_cursor_pos: Option<Vec2>,
}

fn gather_scroll_panels_context(
    panel_query: &Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
    state: &EditorState,
) -> ScrollPanelsContext {
    let mut plain_panel_size = None;
    let mut processed_panel_size = None;
    let mut hovered_panel = None;
    let mut processed_cursor_pos = None;

    for (panel, relative_cursor, computed) in panel_query.iter() {
        if !state.panel_visible(panel.kind) {
            continue;
        }
        let logical_size = computed.size() * computed.inverse_scale_factor();
        match panel.kind {
            PanelKind::Plain => plain_panel_size = Some(logical_size),
            PanelKind::Processed => {
                processed_panel_size = Some(logical_size);
                if relative_cursor.cursor_over() {
                    processed_cursor_pos = relative_cursor.normalized.map(|normalized| {
                        Vec2::new(
                            (normalized.x + 0.5) * logical_size.x,
                            (normalized.y + 0.5) * logical_size.y,
                        )
                    });
                }
            }
        }
        if relative_cursor.cursor_over() {
            hovered_panel = Some(panel.kind);
        }
    }

    ScrollPanelsContext {
        plain_panel_size,
        processed_panel_size,
        hovered_panel,
        processed_cursor_pos,
    }
}
