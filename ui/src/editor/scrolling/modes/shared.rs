#[derive(Clone, Copy)]
struct ScrollPanelsContext {
    plain_panel_size: Option<Vec2>,
    processed_panel_size: Option<Vec2>,
    hovered_panel: Option<PanelKind>,
}

fn gather_scroll_panels_context(
    panel_query: &Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
) -> ScrollPanelsContext {
    let mut plain_panel_size = None;
    let mut processed_panel_size = None;
    let mut hovered_panel = None;

    for (panel, relative_cursor, computed) in panel_query.iter() {
        let logical_size = computed.size() * computed.inverse_scale_factor();
        match panel.kind {
            PanelKind::Plain => plain_panel_size = Some(logical_size),
            PanelKind::Processed => processed_panel_size = Some(logical_size),
        }
        if relative_cursor.cursor_over() {
            hovered_panel = Some(panel.kind);
        }
    }

    ScrollPanelsContext {
        plain_panel_size,
        processed_panel_size,
        hovered_panel,
    }
}
