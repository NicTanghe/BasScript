const STATUS_LINE_FONT_SIZE: f32 = 11.0;
const STATUS_LINE_PADDING_LEFT: f32 = 18.0;
const STATUS_LINE_PADDING_RIGHT: f32 = 0.0;
const STATUS_LINE_PADDING_TOP: f32 = 4.0;
const STATUS_LINE_PADDING_BOTTOM: f32 = 0.0;
const STATUS_LINE_LINE_HEIGHT: f32 = 11.0;

#[derive(Component)]
struct StatusText;

fn status_path_label(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "<unnamed>".to_string())
}

fn status_line_bundle(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            padding: UiRect::new(
                px(STATUS_LINE_PADDING_LEFT),
                px(STATUS_LINE_PADDING_RIGHT),
                px(STATUS_LINE_PADDING_TOP),
                px(STATUS_LINE_PADDING_BOTTOM),
            ),
            overflow: Overflow::clip(),
            ..default()
        },
        children![(
            Node {
                width: percent(100.0),
                overflow: Overflow::clip(),
                ..default()
            },
            Text::new(""),
            TextLayout::new_with_no_wrap(),
            TextFont {
                font,
                font_size: STATUS_LINE_FONT_SIZE,
                ..default()
            },
            LineHeight::Px(STATUS_LINE_LINE_HEIGHT),
            TextColor(COLOR_TEXT_MAIN),
            StatusText,
        )],
    )
}

impl EditorState {
    fn visible_status(&self) -> String {
        format!(
            "{} | format: {} | line {}, col {} | load: {} | save: {}",
            self.status_message,
            document_format_label(self.document_format),
            self.cursor.position.line + 1,
            self.cursor.position.column + 1,
            status_path_label(&self.paths.load_path),
            status_path_label(&self.paths.save_path)
        )
    }
}
