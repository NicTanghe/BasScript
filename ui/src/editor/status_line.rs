const STATUS_LINE_FONT_SIZE: f32 = 11.0;
const STATUS_LINE_PADDING_LEFT: f32 = 18.0;
const STATUS_LINE_PADDING_RIGHT: f32 = 0.0;
const STATUS_LINE_PADDING_TOP: f32 = 4.0;
const STATUS_LINE_PADDING_BOTTOM: f32 = 0.0;
const STATUS_LINE_LINE_HEIGHT: f32 = 11.0;
const STATUS_LINE_MIN_HEIGHT: f32 =
    STATUS_LINE_PADDING_TOP + STATUS_LINE_LINE_HEIGHT + STATUS_LINE_PADDING_BOTTOM;

#[derive(Component)]
struct StatusText;

fn status_path_label(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "<unnamed>".to_string())
}

fn status_line_bundle(font: Handle<Font>, background: Color) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            min_height: px(STATUS_LINE_MIN_HEIGHT),
            flex_shrink: 0.0,
            padding: UiRect::new(
                px(STATUS_LINE_PADDING_LEFT),
                px(STATUS_LINE_PADDING_RIGHT),
                px(STATUS_LINE_PADDING_TOP),
                px(STATUS_LINE_PADDING_BOTTOM),
            ),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(background),
        StatusLineRoot,
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
        let vim_label = if self.vim_enabled {
            format!(" | vim: {}", self.vim_mode.label())
        } else {
            String::new()
        };
        let command_label = if self.command_menu.is_some() {
            " | command".to_string()
        } else {
            String::new()
        };
        let story_index_label = self.story_index_visible_label();

        if self.document_format == DocumentFormat::Canvas {
            let canvas_label = self.canvas_document.as_ref().map_or_else(
                || {
                    self.canvas_parse_error
                        .as_deref()
                        .unwrap_or("invalid canvas")
                        .to_string()
                },
                |canvas| format!("{} nodes, {} edges", canvas.nodes.len(), canvas.edges.len()),
            );
            return format!(
                "{}{}{}{} | format: {} | {} | load: {} | save: {}",
                self.status_message,
                vim_label,
                command_label,
                story_index_label,
                document_format_label(self.document_format),
                canvas_label,
                status_path_label(&self.paths.load_path),
                status_path_label(&self.paths.save_path)
            );
        }

        format!(
            "{}{}{}{} | format: {} | line {}, col {} | load: {} | save: {}",
            self.status_message,
            vim_label,
            command_label,
            story_index_label,
            document_format_label(self.document_format),
            self.cursor.position.line + 1,
            self.cursor.position.column + 1,
            status_path_label(&self.paths.load_path),
            status_path_label(&self.paths.save_path)
        )
    }
}
