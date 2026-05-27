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
}

fn canvas_node_size(width: f32, height: f32) -> Vec2 {
    Vec2::new(
        width.max(CANVAS_NODE_DEFAULT_WIDTH),
        height.max(CANVAS_NODE_DEFAULT_HEIGHT),
    )
}
