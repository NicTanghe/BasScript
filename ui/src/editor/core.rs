use std::{
    collections::BTreeMap,
    fs, io,
    path::PathBuf,
    time::{Duration, Instant},
};

use basscript_core::{
    Cursor, Document, DocumentPath, LineKind, ParsedLine, Position, parse_document,
};
use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        mouse::{MouseScrollUnit, MouseWheel},
    },
    log::{info, warn},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures_lite::future},
    text::{LineHeight, TextLayoutInfo},
    ui::{RelativeCursorPosition, UiTransform, Val2},
    window::{PrimaryWindow, RawHandleWrapper},
};
use rfd::AsyncFileDialog;

const FONT_PATH: &str = "fonts/Courier Prime/Courier Prime.ttf";
const FONT_BOLD_PATH: &str = "fonts/Courier Prime/Courier Prime Bold.ttf";
const FONT_ITALIC_PATH: &str = "fonts/Courier Prime/Courier Prime Italic.ttf";
const FONT_BOLD_ITALIC_PATH: &str = "fonts/Courier Prime/Courier Prime Bold Italic.ttf";
const DEFAULT_LOAD_PATH: &str = "docs/humanDOC.md";
const DEFAULT_SAVE_PATH: &str = "scripts/session.fountain";
const SETTINGS_PATH: &str = "scripts/settings.toml";
const PROCESSED_PAPER_CAPACITY: usize = 16;

const FONT_SIZE: f32 = 12.0;
const LINE_HEIGHT: f32 = 12.0;
const DEFAULT_CHAR_WIDTH: f32 = 7.2;
const TEXT_PADDING_X: f32 = 14.0;
const TEXT_PADDING_Y: f32 = 10.0;
const CARET_WIDTH: f32 = 2.0;
const CARET_X_OFFSET: f32 = -1.0;
const CARET_Y_OFFSET_FACTOR: f32 = -0.12;
const ZOOM_MIN: f32 = 0.6;
const ZOOM_MAX: f32 = 1.8;
const ZOOM_STEP: f32 = 0.1;
const HISTORY_LIMIT: usize = 512;
const MM_PER_INCH: f32 = 25.4;
const POINTS_PER_INCH: f32 = 72.0;
const A4_WIDTH_MM: f32 = 210.0;
const A4_HEIGHT_MM: f32 = 297.0;
const A4_WIDTH_POINTS: f32 = A4_WIDTH_MM / MM_PER_INCH * POINTS_PER_INCH;
const A4_HEIGHT_POINTS: f32 = A4_HEIGHT_MM / MM_PER_INCH * POINTS_PER_INCH;
const PAGE_OUTER_MARGIN: f32 = 14.0;
const PAGE_TEXT_MARGIN_LEFT: f32 = 42.0;
const PAGE_TEXT_MARGIN_RIGHT: f32 = 34.0;
const PAGE_TEXT_MARGIN_TOP: f32 = 30.0;
const PAGE_TEXT_MARGIN_BOTTOM: f32 = 30.0;
const PAGE_GAP: f32 = 24.0;
const PAGE_MARGIN_STEP: f32 = 8.0;
const MIN_TEXT_BOX_WIDTH: f32 = 120.0;
const MIN_TEXT_BOX_HEIGHT: f32 = 120.0;

const BUTTON_NORMAL: Color = Color::srgb(0.80, 0.82, 0.84);
const BUTTON_HOVER: Color = Color::srgb(0.74, 0.77, 0.80);
const BUTTON_PRESSED: Color = Color::srgb(0.68, 0.72, 0.76);
const COLOR_ACTION: Color = Color::srgb(0.12, 0.13, 0.15);
const COLOR_SCENE: Color = Color::srgb(0.10, 0.10, 0.12);
const COLOR_CHARACTER: Color = Color::srgb(0.20, 0.16, 0.12);
const COLOR_DIALOGUE: Color = Color::srgb(0.11, 0.12, 0.13);
const COLOR_PARENTHETICAL: Color = Color::srgb(0.24, 0.28, 0.32);
const COLOR_TRANSITION: Color = Color::srgb(0.15, 0.23, 0.31);
const COLOR_APP_BG: Color = Color::srgb(0.79, 0.80, 0.82);
const COLOR_PANEL_BG: Color = Color::srgb(0.89, 0.90, 0.91);
const COLOR_PANEL_BODY_PLAIN: Color = Color::srgb(0.96, 0.96, 0.97);
const COLOR_PANEL_BODY_PROCESSED: Color = Color::srgb(0.82, 0.83, 0.84);
const COLOR_PAPER: Color = Color::srgb(1.0, 1.0, 1.0);
const COLOR_TEXT_MAIN: Color = Color::srgb(0.18, 0.19, 0.20);
const COLOR_TEXT_MUTED: Color = Color::srgb(0.34, 0.36, 0.39);

pub struct UiPlugin;

#[derive(States, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
enum UiScreenState {
    #[default]
    Editor,
    Settings,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorState>()
            .init_resource::<DialogState>()
            .init_state::<UiScreenState>()
            .insert_non_send_resource(DialogMainThreadMarker)
            .add_systems(Startup, (setup, setup_processed_papers.after(setup)))
            .add_systems(Update, (style_toolbar_buttons, sync_settings_ui))
            .add_systems(
                Update,
                handle_toolbar_buttons.run_if(in_state(UiScreenState::Editor)),
            )
            .add_systems(
                Update,
                handle_settings_buttons.run_if(in_state(UiScreenState::Settings)),
            )
            .add_systems(
                Update,
                (
                    handle_file_shortcuts,
                    resolve_dialog_results,
                    handle_text_input,
                    handle_navigation_input,
                    handle_mouse_scroll,
                    handle_mouse_click,
                    blink_caret,
                    render_editor,
                )
                    .run_if(in_state(UiScreenState::Editor)),
            );
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum PanelKind {
    Plain,
    Processed,
}

#[derive(Component)]
struct PanelBody {
    kind: PanelKind,
}

#[derive(Component)]
struct PanelText {
    kind: PanelKind,
}

#[derive(Component)]
struct PanelCaret {
    kind: PanelKind,
}

#[derive(Component)]
struct PanelPaper {
    kind: PanelKind,
    slot: usize,
}

#[derive(Component)]
struct PanelCanvas {
    kind: PanelKind,
}

#[derive(Component)]
struct ProcessedPaperText {
    slot: usize,
}

#[derive(Component)]
struct ProcessedPaperLineSpan {
    slot: usize,
    line_offset: usize,
}

#[derive(Component)]
struct StatusText;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum ToolbarAction {
    Load,
    SaveAs,
    ZoomOut,
    ZoomIn,
    Settings,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsAction {
    DialogueDoubleSpaceNewline,
    NonDialogueDoubleSpaceNewline,
    MarginLeftDecrease,
    MarginLeftIncrease,
    MarginRightDecrease,
    MarginRightIncrease,
    MarginTopDecrease,
    MarginTopIncrease,
    MarginBottomDecrease,
    MarginBottomIncrease,
    BackToEditor,
}

#[derive(Component)]
struct EditorScreenRoot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct SettingToggleLabel {
    action: SettingsAction,
}

#[derive(Component)]
struct SettingsScreenRoot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum MarginEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct SettingMarginLabel {
    edge: MarginEdge,
}

#[derive(Resource)]
struct EditorState {
    document: Document,
    parsed: Vec<ParsedLine>,
    cursor: Cursor,
    top_line: usize,
    plain_horizontal_scroll: f32,
    processed_horizontal_scroll: f32,
    paths: DocumentPath,
    status_message: String,
    caret_blink: Timer,
    caret_visible: bool,
    dialogue_double_space_newline: bool,
    non_dialogue_double_space_newline: bool,
    page_margin_left: f32,
    page_margin_right: f32,
    page_margin_top: f32,
    page_margin_bottom: f32,
    zoom: f32,
    measured_line_step: f32,
    processed_cache: Option<ProcessedCache>,
    processed_cache_dirty_from_line: Option<usize>,
    undo_history: Vec<EditorHistorySnapshot>,
    redo_history: Vec<EditorHistorySnapshot>,
}

#[derive(Clone)]
struct EditorHistorySnapshot {
    document: Document,
    cursor: Cursor,
    top_line: usize,
    plain_horizontal_scroll: f32,
    processed_horizontal_scroll: f32,
}

#[derive(Resource, Default)]
struct DialogState {
    pending: Option<PendingDialog>,
    opened_at: Option<Instant>,
    last_watchdog_log_at: Option<Instant>,
    poll_count: u64,
}

enum PendingDialog {
    Load(Task<Option<PathBuf>>),
    Save(Task<Option<PathBuf>>),
}

struct DialogMainThreadMarker;

#[derive(Clone, Copy, Debug)]
struct PersistentSettings {
    dialogue_double_space_newline: bool,
    non_dialogue_double_space_newline: bool,
    page_margin_left: f32,
    page_margin_right: f32,
    page_margin_top: f32,
    page_margin_bottom: f32,
}

impl Default for PersistentSettings {
    fn default() -> Self {
        Self {
            dialogue_double_space_newline: false,
            non_dialogue_double_space_newline: false,
            page_margin_left: PAGE_TEXT_MARGIN_LEFT,
            page_margin_right: PAGE_TEXT_MARGIN_RIGHT,
            page_margin_top: PAGE_TEXT_MARGIN_TOP,
            page_margin_bottom: PAGE_TEXT_MARGIN_BOTTOM,
        }
    }
}

#[derive(Resource, Clone)]
struct EditorFonts {
    regular: Handle<Font>,
    bold: Handle<Font>,
    italic: Handle<Font>,
    bold_italic: Handle<Font>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FontVariant {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

impl DialogState {
    fn begin_pending(&mut self, pending: PendingDialog) {
        let now = Instant::now();
        self.pending = Some(pending);
        self.opened_at = Some(now);
        self.last_watchdog_log_at = Some(now);
        self.poll_count = 0;
    }

    fn clear_pending(&mut self) {
        self.pending = None;
        self.opened_at = None;
        self.last_watchdog_log_at = None;
        self.poll_count = 0;
    }
}

impl PendingDialog {
    fn kind_name(&self) -> &'static str {
        match self {
            PendingDialog::Load(_) => "load",
            PendingDialog::Save(_) => "save",
        }
    }
}

impl FromWorld for EditorState {
    fn from_world(_world: &mut World) -> Self {
        let paths = DocumentPath::new(DEFAULT_LOAD_PATH, DEFAULT_SAVE_PATH);
        let settings = load_persistent_settings();

        let (document, status_message) = match Document::load(&paths.load_path) {
            Ok(doc) => (doc, format!("Loaded {}", paths.load_path.display())),
            Err(error) => (
                Document::new(),
                format!(
                    "Could not load {} ({error}). Started empty document.",
                    paths.load_path.display()
                ),
            ),
        };

        let parsed = parse_document(&document);

        let mut next = Self {
            document,
            parsed,
            cursor: Cursor::default(),
            top_line: 0,
            plain_horizontal_scroll: 0.0,
            processed_horizontal_scroll: 0.0,
            paths,
            status_message,
            caret_blink: Timer::from_seconds(0.5, TimerMode::Repeating),
            caret_visible: true,
            dialogue_double_space_newline: settings.dialogue_double_space_newline,
            non_dialogue_double_space_newline: settings.non_dialogue_double_space_newline,
            page_margin_left: settings.page_margin_left,
            page_margin_right: settings.page_margin_right,
            page_margin_top: settings.page_margin_top,
            page_margin_bottom: settings.page_margin_bottom,
            zoom: 1.0,
            measured_line_step: LINE_HEIGHT,
            processed_cache: None,
            processed_cache_dirty_from_line: Some(0),
            undo_history: Vec::new(),
            redo_history: Vec::new(),
        };
        normalize_page_margins(&mut next);
        next
    }
}

impl EditorState {
    fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(ZOOM_MIN, ZOOM_MAX);
        self.measured_line_step = scaled_line_height(self);
        self.reset_blink();
    }

    fn zoom_percent(&self) -> u32 {
        (self.zoom * 100.0).round() as u32
    }

    fn reparse(&mut self) {
        self.parsed = parse_document(&self.document);
        self.mark_processed_cache_dirty_from(0);
    }

    fn reparse_with_dirty_hint(&mut self, dirty_line: usize) {
        self.parsed = parse_document(&self.document);
        self.mark_processed_cache_dirty_from(dirty_line);
    }

    fn mark_processed_cache_dirty_from(&mut self, source_line: usize) {
        let dirty_line = source_line.min(self.document.line_count().saturating_sub(1));
        self.processed_cache_dirty_from_line = Some(
            self.processed_cache_dirty_from_line
                .map_or(dirty_line, |current| current.min(dirty_line)),
        );
    }

    fn reset_blink(&mut self) {
        self.caret_blink.reset();
        self.caret_visible = true;
    }

    fn visible_status(&self) -> String {
        format!(
            "{} | line {}, col {} | load: {} | save: {}",
            self.status_message,
            self.cursor.position.line + 1,
            self.cursor.position.column + 1,
            self.paths.load_path.display(),
            self.paths.save_path.display()
        )
    }

    fn max_top_line(&self, visible_lines: usize) -> usize {
        self.document
            .line_count()
            .saturating_sub(visible_lines.max(1))
    }

    fn clamp_scroll(&mut self, visible_lines: usize) {
        let max_top = self.max_top_line(visible_lines);
        self.top_line = self.top_line.min(max_top);
    }

    fn clamp_horizontal_scrolls(
        &mut self,
        plain_panel_size: Option<Vec2>,
        processed_panel_size: Option<Vec2>,
    ) {
        let plain_max = plain_horizontal_scroll_max(self, plain_panel_size);
        self.plain_horizontal_scroll = self.plain_horizontal_scroll.clamp(0.0, plain_max);

        let (processed_min, processed_max) =
            processed_horizontal_scroll_bounds(self, processed_panel_size);
        self.processed_horizontal_scroll = self
            .processed_horizontal_scroll
            .clamp(processed_min, processed_max);
    }

    fn scroll_by(&mut self, line_delta: isize, visible_lines: usize) {
        let max_top = self.max_top_line(visible_lines) as isize;
        let next = (self.top_line as isize + line_delta).clamp(0, max_top);
        self.top_line = next as usize;
    }

    fn ensure_cursor_visible(&mut self, visible_lines: usize) {
        if self.cursor.position.line < self.top_line {
            self.top_line = self.cursor.position.line;
        } else if self.cursor.position.line >= self.top_line + visible_lines {
            self.top_line = self
                .cursor
                .position
                .line
                .saturating_sub(visible_lines.saturating_sub(1));
        }

        self.clamp_scroll(visible_lines);
    }

    fn clamp_cursor_to_visible_range(&mut self, visible_lines: usize) {
        if self.document.is_empty() {
            self.set_cursor(Position::default(), true);
            return;
        }

        let min_line = self.top_line;
        let max_line = self
            .top_line
            .saturating_add(visible_lines.saturating_sub(1))
            .min(self.document.line_count().saturating_sub(1));
        let clamped_line = self.cursor.position.line.clamp(min_line, max_line);

        if clamped_line != self.cursor.position.line {
            let column = self
                .cursor
                .preferred_column
                .min(self.document.line_len_chars(clamped_line));
            self.set_cursor(
                Position {
                    line: clamped_line,
                    column,
                },
                false,
            );
        }
    }

    fn set_cursor(&mut self, position: Position, update_preferred: bool) {
        let clamped = self.document.clamp_position(position);

        if update_preferred {
            self.cursor.set_position(clamped);
        } else {
            self.cursor.position = clamped;
        }

        self.reset_blink();
    }

    fn save_to_path(&mut self, path: PathBuf) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        match self.document.save(&path) {
            Ok(()) => {
                self.paths.save_path = path.clone();
                self.status_message = format!("Saved {}", path.display());
            }
            Err(error) => {
                self.status_message = format!("Save failed for {}: {error}", path.display());
            }
        }
    }

    fn load_from_path(&mut self, path: PathBuf) {
        match Document::load(&path) {
            Ok(document) => {
                self.document = document;
                self.reparse();
                self.cursor = Cursor::default();
                self.top_line = 0;
                self.plain_horizontal_scroll = 0.0;
                self.processed_horizontal_scroll = 0.0;
                self.clear_history();
                self.paths.load_path = path.clone();
                self.paths.save_path = path.clone();
                self.status_message = format!("Loaded {}", path.display());
                self.reset_blink();
            }
            Err(error) => {
                self.status_message = format!("Load failed for {}: {error}", path.display());
            }
        }
    }

    fn history_snapshot(&self) -> EditorHistorySnapshot {
        EditorHistorySnapshot {
            document: self.document.clone(),
            cursor: self.cursor,
            top_line: self.top_line,
            plain_horizontal_scroll: self.plain_horizontal_scroll,
            processed_horizontal_scroll: self.processed_horizontal_scroll,
        }
    }

    fn push_history_snapshot(
        history: &mut Vec<EditorHistorySnapshot>,
        snapshot: EditorHistorySnapshot,
    ) {
        if history.len() >= HISTORY_LIMIT {
            history.remove(0);
        }
        history.push(snapshot);
    }

    fn push_undo_snapshot(&mut self, snapshot: EditorHistorySnapshot) {
        Self::push_history_snapshot(&mut self.undo_history, snapshot);
        self.redo_history.clear();
    }

    fn apply_history_snapshot(
        &mut self,
        snapshot: EditorHistorySnapshot,
        visible_lines: usize,
        plain_panel_size: Option<Vec2>,
        processed_panel_size: Option<Vec2>,
    ) {
        self.document = snapshot.document;
        self.parsed = parse_document(&self.document);
        self.processed_cache = None;
        self.processed_cache_dirty_from_line = Some(0);

        self.cursor = snapshot.cursor;
        self.cursor.position = self.document.clamp_position(self.cursor.position);
        self.cursor.preferred_column = self
            .cursor
            .preferred_column
            .min(self.document.line_len_chars(self.cursor.position.line));

        self.top_line = snapshot.top_line;
        self.plain_horizontal_scroll = snapshot.plain_horizontal_scroll;
        self.processed_horizontal_scroll = snapshot.processed_horizontal_scroll;
        self.clamp_scroll(visible_lines);
        self.clamp_horizontal_scrolls(plain_panel_size, processed_panel_size);
        self.reset_blink();
    }

    fn undo(
        &mut self,
        visible_lines: usize,
        plain_panel_size: Option<Vec2>,
        processed_panel_size: Option<Vec2>,
    ) -> bool {
        let Some(snapshot) = self.undo_history.pop() else {
            return false;
        };

        let current = self.history_snapshot();
        Self::push_history_snapshot(&mut self.redo_history, current);
        self.apply_history_snapshot(
            snapshot,
            visible_lines,
            plain_panel_size,
            processed_panel_size,
        );
        true
    }

    fn redo(
        &mut self,
        visible_lines: usize,
        plain_panel_size: Option<Vec2>,
        processed_panel_size: Option<Vec2>,
    ) -> bool {
        let Some(snapshot) = self.redo_history.pop() else {
            return false;
        };

        let current = self.history_snapshot();
        Self::push_history_snapshot(&mut self.undo_history, current);
        self.apply_history_snapshot(
            snapshot,
            visible_lines,
            plain_panel_size,
            processed_panel_size,
        );
        true
    }

    fn clear_history(&mut self) {
        self.undo_history.clear();
        self.redo_history.clear();
    }
}

#[derive(Clone, Copy, Debug)]
struct ProcessedPageGeometry {
    paper_left: f32,
    paper_top: f32,
    paper_width: f32,
    paper_height: f32,
    text_left: f32,
    text_top: f32,
    text_width: f32,
    text_height: f32,
}

#[derive(Clone, Copy, Debug)]
struct ProcessedPageLayout {
    geometry: ProcessedPageGeometry,
    wrap_columns: usize,
    lines_per_page: usize,
    spacer_lines: usize,
    page_step_lines: usize,
}
