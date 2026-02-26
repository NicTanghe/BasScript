use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use basscript_core::{
    Cursor, Document, DocumentFormat, DocumentPath, LineKind, ParsedLine, Position,
    parse_document_with_format,
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
const FONT_MARKDOWN_PATH: &str = "fonts/SegoeUIVF.ttf";
const FONT_MARKDOWN_BOLD_PATH: &str = "fonts/SegoeUIVF.ttf";
const FONT_MARKDOWN_ITALIC_PATH: &str = "fonts/SegoeUIVF.ttf";
const FONT_MARKDOWN_BOLD_ITALIC_PATH: &str = "fonts/SegoeUIVF.ttf";
const DEFAULT_LOAD_PATH: &str = "docs/humanDOC.md";
const DEFAULT_SAVE_PATH: &str = "scripts/session.fountain";
const EDITOR_SETTINGS_PATH: &str = "scripts/editor_settings.ron";
const KEYBINDS_SETTINGS_PATH: &str = "scripts/keybinds.ron";
const LEGACY_SETTINGS_PATH: &str = "scripts/settings.toml";
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
const COLOR_MARKDOWN_HEADING: Color = Color::srgb(0.18, 0.24, 0.40);
const COLOR_MARKDOWN_LIST: Color = Color::srgb(0.16, 0.22, 0.31);
const COLOR_MARKDOWN_QUOTE: Color = Color::srgb(0.22, 0.29, 0.26);
const COLOR_MARKDOWN_CODE: Color = Color::srgb(0.29, 0.17, 0.18);
const COLOR_MARKDOWN_RULE: Color = Color::srgb(0.35, 0.35, 0.38);
const COLOR_APP_BG: Color = Color::srgb(0.79, 0.80, 0.82);
const COLOR_PANEL_BG: Color = Color::srgb(0.89, 0.90, 0.91);
const COLOR_PANEL_BODY_PLAIN: Color = Color::srgb(0.96, 0.96, 0.97);
const COLOR_PANEL_BODY_PROCESSED: Color = Color::srgb(0.82, 0.83, 0.84);
const COLOR_PAPER: Color = Color::srgb(1.0, 1.0, 1.0);
const COLOR_TEXT_MAIN: Color = Color::srgb(0.18, 0.19, 0.20);
const COLOR_TEXT_MUTED: Color = Color::srgb(0.34, 0.36, 0.39);
const COLOR_WORKSPACE_FILE: Color = Color::srgb(0.18, 0.19, 0.20);
const COLOR_WORKSPACE_FILE_HOVER: Color = Color::srgb(0.10, 0.35, 0.62);
const COLOR_WORKSPACE_FILE_SELECTED: Color = Color::srgb(0.69, 0.28, 0.22);

pub struct UiPlugin;

#[derive(States, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
enum UiScreenState {
    #[default]
    Editor,
    Settings,
    Keybinds,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorState>()
            .init_resource::<DialogState>()
            .init_resource::<MiddleAutoscrollState>()
            .init_state::<UiScreenState>()
            .insert_non_send_resource(DialogMainThreadMarker)
            .add_systems(Startup, (setup, setup_processed_papers.after(setup)))
            .add_systems(
                Update,
                (
                    style_toolbar_buttons,
                    style_workspace_file_entry_text,
                    handle_window_shortcuts,
                    sync_window_chrome,
                    sync_top_menu_visibility,
                    sync_panel_display_mode,
                    sync_settings_ui,
                    sync_workspace_sidebar,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_toolbar_buttons,
                    handle_workspace_file_buttons,
                    handle_workspace_folder_buttons,
                )
                    .run_if(in_state(UiScreenState::Editor)),
            )
            .add_systems(
                Update,
                handle_settings_buttons
                    .run_if(in_state(UiScreenState::Settings).or(in_state(UiScreenState::Keybinds))),
            )
            .add_systems(
                Update,
                (handle_keybind_buttons, capture_keybind_input)
                    .run_if(in_state(UiScreenState::Keybinds)),
            )
            .add_systems(
                Update,
                (
                    handle_file_shortcuts,
                    resolve_dialog_results,
                    handle_text_input,
                    handle_navigation_input,
                    handle_mouse_scroll,
                    handle_ctrl_left_drag_scroll,
                    handle_middle_mouse_autoscroll,
                    handle_mouse_click.after(handle_middle_mouse_autoscroll),
                    sync_middle_autoscroll_indicator.after(handle_middle_mouse_autoscroll),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DisplayMode {
    Split,
    Plain,
    Processed,
    ProcessedRawCurrentLine,
}

impl DisplayMode {
    fn label(self) -> &'static str {
        match self {
            DisplayMode::Split => "Split",
            DisplayMode::Plain => "Plain",
            DisplayMode::Processed => "Processed",
            DisplayMode::ProcessedRawCurrentLine => "Processed + Raw Line",
        }
    }

    fn panel_visible(self, panel: PanelKind) -> bool {
        match self {
            DisplayMode::Split => true,
            DisplayMode::Plain => panel == PanelKind::Plain,
            DisplayMode::Processed | DisplayMode::ProcessedRawCurrentLine => {
                panel == PanelKind::Processed
            }
        }
    }
}

#[derive(Component)]
struct PanelRoot {
    kind: PanelKind,
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
struct MiddleAutoscrollIndicator;

#[derive(Component)]
struct ProcessedPaperText {
    slot: usize,
}

#[derive(Component)]
struct ProcessedPaperLineSpan {
    slot: usize,
    line_offset: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct ProcessedChecklistIcon {
    slot: usize,
    line_offset: usize,
}

#[derive(Component)]
struct WorkspaceRootLabel;

#[derive(Component)]
struct WorkspaceFileList;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct WorkspaceFileButton {
    index: usize,
}

#[derive(Component, Clone, Debug, PartialEq, Eq)]
struct WorkspaceFolderToggleButton {
    folder_key: String,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum ToolbarAction {
    OpenWorkspace,
    SaveAs,
    ZoomOut,
    ZoomIn,
    Settings,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsAction {
    DialogueDoubleSpaceNewline,
    NonDialogueDoubleSpaceNewline,
    ShowSystemTitlebar,
    MarginLeftDecrease,
    MarginLeftIncrease,
    MarginRightDecrease,
    MarginRightIncrease,
    MarginTopDecrease,
    MarginTopIncrease,
    MarginBottomDecrease,
    MarginBottomIncrease,
    OpenKeybinds,
    BackToSettings,
    BackToEditor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ShortcutAction {
    OpenWorkspace,
    SaveAs,
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    PlainView,
    ProcessedView,
    ProcessedRawCurrentLineView,
    ToggleTopMenu,
}

const SHORTCUT_ACTIONS: [ShortcutAction; 10] = [
    ShortcutAction::OpenWorkspace,
    ShortcutAction::SaveAs,
    ShortcutAction::Undo,
    ShortcutAction::Redo,
    ShortcutAction::ZoomIn,
    ShortcutAction::ZoomOut,
    ShortcutAction::PlainView,
    ShortcutAction::ProcessedView,
    ShortcutAction::ProcessedRawCurrentLineView,
    ShortcutAction::ToggleTopMenu,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ShortcutBinding {
    key: KeyCode,
    shift: bool,
}

#[derive(Clone, Debug)]
struct KeybindSettings {
    open_workspace: ShortcutBinding,
    save_as: ShortcutBinding,
    undo: ShortcutBinding,
    redo: ShortcutBinding,
    zoom_in: ShortcutBinding,
    zoom_out: ShortcutBinding,
    plain_view: ShortcutBinding,
    processed_view: ShortcutBinding,
    processed_raw_current_line_view: ShortcutBinding,
    toggle_top_menu: ShortcutBinding,
}

impl Default for KeybindSettings {
    fn default() -> Self {
        Self {
            open_workspace: ShortcutBinding {
                key: KeyCode::KeyO,
                shift: false,
            },
            save_as: ShortcutBinding {
                key: KeyCode::KeyS,
                shift: false,
            },
            undo: ShortcutBinding {
                key: KeyCode::KeyZ,
                shift: false,
            },
            redo: ShortcutBinding {
                key: KeyCode::KeyZ,
                shift: true,
            },
            zoom_in: ShortcutBinding {
                key: KeyCode::Equal,
                shift: false,
            },
            zoom_out: ShortcutBinding {
                key: KeyCode::Minus,
                shift: false,
            },
            plain_view: ShortcutBinding {
                key: KeyCode::KeyT,
                shift: false,
            },
            processed_view: ShortcutBinding {
                key: KeyCode::KeyR,
                shift: false,
            },
            processed_raw_current_line_view: ShortcutBinding {
                key: KeyCode::Digit1,
                shift: false,
            },
            toggle_top_menu: ShortcutBinding {
                key: KeyCode::KeyB,
                shift: false,
            },
        }
    }
}

impl KeybindSettings {
    fn binding(&self, action: ShortcutAction) -> ShortcutBinding {
        match action {
            ShortcutAction::OpenWorkspace => self.open_workspace,
            ShortcutAction::SaveAs => self.save_as,
            ShortcutAction::Undo => self.undo,
            ShortcutAction::Redo => self.redo,
            ShortcutAction::ZoomIn => self.zoom_in,
            ShortcutAction::ZoomOut => self.zoom_out,
            ShortcutAction::PlainView => self.plain_view,
            ShortcutAction::ProcessedView => self.processed_view,
            ShortcutAction::ProcessedRawCurrentLineView => self.processed_raw_current_line_view,
            ShortcutAction::ToggleTopMenu => self.toggle_top_menu,
        }
    }

    fn set_binding(&mut self, action: ShortcutAction, binding: ShortcutBinding) {
        match action {
            ShortcutAction::OpenWorkspace => self.open_workspace = binding,
            ShortcutAction::SaveAs => self.save_as = binding,
            ShortcutAction::Undo => self.undo = binding,
            ShortcutAction::Redo => self.redo = binding,
            ShortcutAction::ZoomIn => self.zoom_in = binding,
            ShortcutAction::ZoomOut => self.zoom_out = binding,
            ShortcutAction::PlainView => self.plain_view = binding,
            ShortcutAction::ProcessedView => self.processed_view = binding,
            ShortcutAction::ProcessedRawCurrentLineView => {
                self.processed_raw_current_line_view = binding
            }
            ShortcutAction::ToggleTopMenu => self.toggle_top_menu = binding,
        }
    }
}

fn shortcut_action_label(action: ShortcutAction) -> &'static str {
    match action {
        ShortcutAction::OpenWorkspace => "Open Workspace Folder",
        ShortcutAction::SaveAs => "Save As Dialog",
        ShortcutAction::Undo => "Undo",
        ShortcutAction::Redo => "Redo",
        ShortcutAction::ZoomIn => "Zoom In",
        ShortcutAction::ZoomOut => "Zoom Out",
        ShortcutAction::PlainView => "Plain View Mode",
        ShortcutAction::ProcessedView => "Processed View Mode",
        ShortcutAction::ProcessedRawCurrentLineView => "Processed + Raw Current Line Mode",
        ShortcutAction::ToggleTopMenu => "Toggle Top Menu",
    }
}

fn shortcut_action_description(action: ShortcutAction) -> &'static str {
    match action {
        ShortcutAction::OpenWorkspace => "Open workspace folder",
        ShortcutAction::SaveAs => "Save As dialog",
        ShortcutAction::Undo => "Undo",
        ShortcutAction::Redo => "Redo",
        ShortcutAction::ZoomIn => "Zoom in",
        ShortcutAction::ZoomOut => "Zoom out",
        ShortcutAction::PlainView => "Plain view mode",
        ShortcutAction::ProcessedView => "Processed view mode",
        ShortcutAction::ProcessedRawCurrentLineView => "Processed + raw current line mode",
        ShortcutAction::ToggleTopMenu => "Toggle top menu",
    }
}

fn shortcut_action_settings_key(action: ShortcutAction) -> &'static str {
    match action {
        ShortcutAction::OpenWorkspace => "open_workspace",
        ShortcutAction::SaveAs => "save_as",
        ShortcutAction::Undo => "undo",
        ShortcutAction::Redo => "redo",
        ShortcutAction::ZoomIn => "zoom_in",
        ShortcutAction::ZoomOut => "zoom_out",
        ShortcutAction::PlainView => "plain_view",
        ShortcutAction::ProcessedView => "processed_view",
        ShortcutAction::ProcessedRawCurrentLineView => "processed_raw_current_line_view",
        ShortcutAction::ToggleTopMenu => "toggle_top_menu",
    }
}

fn binding_key_name(key: KeyCode) -> Option<&'static str> {
    match key {
        KeyCode::KeyA => Some("A"),
        KeyCode::KeyB => Some("B"),
        KeyCode::KeyC => Some("C"),
        KeyCode::KeyD => Some("D"),
        KeyCode::KeyE => Some("E"),
        KeyCode::KeyF => Some("F"),
        KeyCode::KeyG => Some("G"),
        KeyCode::KeyH => Some("H"),
        KeyCode::KeyI => Some("I"),
        KeyCode::KeyJ => Some("J"),
        KeyCode::KeyK => Some("K"),
        KeyCode::KeyL => Some("L"),
        KeyCode::KeyM => Some("M"),
        KeyCode::KeyN => Some("N"),
        KeyCode::KeyO => Some("O"),
        KeyCode::KeyP => Some("P"),
        KeyCode::KeyQ => Some("Q"),
        KeyCode::KeyR => Some("R"),
        KeyCode::KeyS => Some("S"),
        KeyCode::KeyT => Some("T"),
        KeyCode::KeyU => Some("U"),
        KeyCode::KeyV => Some("V"),
        KeyCode::KeyW => Some("W"),
        KeyCode::KeyX => Some("X"),
        KeyCode::KeyY => Some("Y"),
        KeyCode::KeyZ => Some("Z"),
        KeyCode::Digit0 | KeyCode::Numpad0 => Some("0"),
        KeyCode::Digit1 | KeyCode::Numpad1 => Some("1"),
        KeyCode::Digit2 | KeyCode::Numpad2 => Some("2"),
        KeyCode::Digit3 | KeyCode::Numpad3 => Some("3"),
        KeyCode::Digit4 | KeyCode::Numpad4 => Some("4"),
        KeyCode::Digit5 | KeyCode::Numpad5 => Some("5"),
        KeyCode::Digit6 | KeyCode::Numpad6 => Some("6"),
        KeyCode::Digit7 | KeyCode::Numpad7 => Some("7"),
        KeyCode::Digit8 | KeyCode::Numpad8 => Some("8"),
        KeyCode::Digit9 | KeyCode::Numpad9 => Some("9"),
        KeyCode::Equal => Some("="),
        KeyCode::Minus => Some("-"),
        _ => None,
    }
}

fn binding_key_from_name(name: &str) -> Option<KeyCode> {
    match name.trim().to_ascii_uppercase().as_str() {
        "A" => Some(KeyCode::KeyA),
        "B" => Some(KeyCode::KeyB),
        "C" => Some(KeyCode::KeyC),
        "D" => Some(KeyCode::KeyD),
        "E" => Some(KeyCode::KeyE),
        "F" => Some(KeyCode::KeyF),
        "G" => Some(KeyCode::KeyG),
        "H" => Some(KeyCode::KeyH),
        "I" => Some(KeyCode::KeyI),
        "J" => Some(KeyCode::KeyJ),
        "K" => Some(KeyCode::KeyK),
        "L" => Some(KeyCode::KeyL),
        "M" => Some(KeyCode::KeyM),
        "N" => Some(KeyCode::KeyN),
        "O" => Some(KeyCode::KeyO),
        "P" => Some(KeyCode::KeyP),
        "Q" => Some(KeyCode::KeyQ),
        "R" => Some(KeyCode::KeyR),
        "S" => Some(KeyCode::KeyS),
        "T" => Some(KeyCode::KeyT),
        "U" => Some(KeyCode::KeyU),
        "V" => Some(KeyCode::KeyV),
        "W" => Some(KeyCode::KeyW),
        "X" => Some(KeyCode::KeyX),
        "Y" => Some(KeyCode::KeyY),
        "Z" => Some(KeyCode::KeyZ),
        "0" => Some(KeyCode::Digit0),
        "1" => Some(KeyCode::Digit1),
        "2" => Some(KeyCode::Digit2),
        "3" => Some(KeyCode::Digit3),
        "4" => Some(KeyCode::Digit4),
        "5" => Some(KeyCode::Digit5),
        "6" => Some(KeyCode::Digit6),
        "7" => Some(KeyCode::Digit7),
        "8" => Some(KeyCode::Digit8),
        "9" => Some(KeyCode::Digit9),
        "=" => Some(KeyCode::Equal),
        "-" => Some(KeyCode::Minus),
        _ => None,
    }
}

fn parse_binding_spec(spec: &str) -> Option<ShortcutBinding> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (shift, key_name) = if trimmed
        .to_ascii_uppercase()
        .starts_with("SHIFT+")
        && trimmed.len() > "SHIFT+".len()
    {
        (true, &trimmed["SHIFT+".len()..])
    } else {
        (false, trimmed)
    };
    let key = binding_key_from_name(key_name)?;
    Some(ShortcutBinding { key, shift })
}

fn binding_spec(binding: ShortcutBinding) -> String {
    let key_name = binding_key_name(binding.key).unwrap_or("?");
    if binding.shift {
        format!("Shift+{key_name}")
    } else {
        key_name.to_string()
    }
}

fn binding_display(binding: ShortcutBinding) -> String {
    let mut text = String::from("Cmd/Ctrl+");
    if binding.shift {
        text.push_str("Shift+");
    }
    text.push_str(binding_key_name(binding.key).unwrap_or("?"));
    text
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct KeybindRebindButton {
    action: ShortcutAction,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct KeybindBindingLabel {
    action: ShortcutAction,
}

#[derive(Component)]
struct EditorScreenRoot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct SettingToggleLabel {
    action: SettingsAction,
}

#[derive(Component)]
struct SettingsScreenRoot;

#[derive(Component)]
struct KeybindsScreenRoot;

#[derive(Component)]
struct TopMenuSection;

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
    document_format: DocumentFormat,
    cursor: Cursor,
    top_line: usize,
    processed_top_line: usize,
    processed_top_visual: usize,
    display_mode: DisplayMode,
    focused_panel: PanelKind,
    plain_horizontal_scroll: f32,
    processed_horizontal_scroll: f32,
    processed_zoom_anchor_bias_px: f32,
    paths: DocumentPath,
    status_message: String,
    keybinds: KeybindSettings,
    pending_keybind_capture: Option<ShortcutAction>,
    top_menu_collapsed: bool,
    show_system_titlebar: bool,
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
    workspace_root: Option<PathBuf>,
    workspace_files: Vec<WorkspaceFileEntry>,
    workspace_selected: Option<usize>,
    workspace_expanded_folders: BTreeSet<String>,
    workspace_ui_dirty: bool,
    undo_history: Vec<EditorHistorySnapshot>,
    redo_history: Vec<EditorHistorySnapshot>,
}

#[derive(Clone)]
struct EditorHistorySnapshot {
    document: Document,
    cursor: Cursor,
    top_line: usize,
    processed_top_line: usize,
    processed_top_visual: usize,
    plain_horizontal_scroll: f32,
    processed_horizontal_scroll: f32,
    processed_zoom_anchor_bias_px: f32,
}

#[derive(Resource, Default)]
struct DialogState {
    pending: Option<PendingDialog>,
    opened_at: Option<Instant>,
    last_watchdog_log_at: Option<Instant>,
    poll_count: u64,
}

#[derive(Resource, Default)]
struct MiddleAutoscrollState {
    panel: Option<PanelKind>,
    anchor_cursor_position: Vec2,
    plain_vertical_remainder_lines: f32,
    suppress_next_left_click: bool,
}

enum PendingDialog {
    Workspace(Task<Option<PathBuf>>),
    Save(Task<Option<PathBuf>>),
}

struct DialogMainThreadMarker;

#[derive(Clone, Copy, Debug)]
struct PersistentSettings {
    dialogue_double_space_newline: bool,
    non_dialogue_double_space_newline: bool,
    show_system_titlebar: bool,
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
            show_system_titlebar: false,
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
    markdown_regular: Handle<Font>,
    markdown_bold: Handle<Font>,
    markdown_italic: Handle<Font>,
    markdown_bold_italic: Handle<Font>,
}

#[derive(Resource, Clone)]
struct WorkspaceIcons {
    folder_closed: Handle<Image>,
    folder_open: Handle<Image>,
}

#[derive(Resource, Clone)]
struct ChecklistIcons {
    unchecked: Handle<Image>,
    checked: Handle<Image>,
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

impl MiddleAutoscrollState {
    fn is_active(&self) -> bool {
        self.panel.is_some()
    }

    fn start(&mut self, panel: PanelKind, anchor_cursor_position: Vec2) {
        self.panel = Some(panel);
        self.anchor_cursor_position = anchor_cursor_position;
        self.plain_vertical_remainder_lines = 0.0;
        self.suppress_next_left_click = false;
    }

    fn stop(&mut self) {
        self.panel = None;
        self.plain_vertical_remainder_lines = 0.0;
    }
}

impl PendingDialog {
    fn kind_name(&self) -> &'static str {
        match self {
            PendingDialog::Workspace(_) => "workspace",
            PendingDialog::Save(_) => "save",
        }
    }
}

impl FromWorld for EditorState {
    fn from_world(_world: &mut World) -> Self {
        let paths = DocumentPath::new(DEFAULT_LOAD_PATH, DEFAULT_SAVE_PATH);
        let settings = load_persistent_settings();
        let keybinds = load_keybind_settings();
        let (document, document_format, status_message) = match Document::load(&paths.load_path) {
            Ok(doc) => {
                let format = detect_document_format(&paths.load_path, &doc);
                (
                    doc,
                    format,
                    format!(
                        "Loaded {} ({}).",
                        status_path_label(&paths.load_path),
                        document_format_label(format)
                    ),
                )
            }
            Err(error) => {
                let doc = Document::new();
                let format = detect_document_format(&paths.load_path, &doc);
                (
                    doc,
                    format,
                    format!(
                        "Could not load {} ({error}). Started empty document.",
                        status_path_label(&paths.load_path)
                    ),
                )
            }
        };

        let parsed = parse_document_with_format(&document, document_format);

        let mut next = Self {
            document,
            parsed,
            document_format,
            cursor: Cursor::default(),
            top_line: 0,
            processed_top_line: 0,
            processed_top_visual: 0,
            display_mode: DisplayMode::Split,
            focused_panel: PanelKind::Plain,
            plain_horizontal_scroll: 0.0,
            processed_horizontal_scroll: 0.0,
            processed_zoom_anchor_bias_px: 0.0,
            paths,
            status_message,
            keybinds,
            pending_keybind_capture: None,
            top_menu_collapsed: false,
            show_system_titlebar: settings.show_system_titlebar,
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
            workspace_root: None,
            workspace_files: Vec::new(),
            workspace_selected: None,
            workspace_expanded_folders: BTreeSet::new(),
            workspace_ui_dirty: true,
            undo_history: Vec::new(),
            redo_history: Vec::new(),
        };
        normalize_page_margins(&mut next);
        let initial_status = next.status_message.clone();
        if let Some(workspace_root) = next.paths.load_path.parent().map(|path| path.to_path_buf()) {
            next.set_workspace_root(workspace_root);
            next.status_message = initial_status;
        }
        next
    }
}

impl EditorState {
    fn set_display_mode(&mut self, mode: DisplayMode) -> bool {
        if self.display_mode == mode {
            return false;
        }

        self.display_mode = mode;
        if !self.panel_visible(self.focused_panel) {
            self.focused_panel = self.active_panel_for_display_mode();
        }
        self.reset_blink();
        true
    }

    fn active_panel_for_display_mode(&self) -> PanelKind {
        match self.display_mode {
            DisplayMode::Split => self.focused_panel,
            DisplayMode::Plain => PanelKind::Plain,
            DisplayMode::Processed | DisplayMode::ProcessedRawCurrentLine => PanelKind::Processed,
        }
    }

    fn panel_visible(&self, panel: PanelKind) -> bool {
        self.display_mode.panel_visible(panel)
    }

    fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(ZOOM_MIN, ZOOM_MAX);
        self.measured_line_step = scaled_line_height(self);
        self.reset_blink();
    }

    fn zoom_percent(&self) -> u32 {
        (self.zoom * 100.0).round() as u32
    }

    fn set_workspace_root(&mut self, root: PathBuf) {
        let normalized_root = root.canonicalize().unwrap_or(root);
        self.workspace_root = Some(normalized_root.clone());

        match collect_workspace_files(&normalized_root) {
            Ok(files) => {
                self.workspace_files = files;
                self.workspace_expanded_folders =
                    default_expanded_workspace_folders(&self.workspace_files);
                self.sync_workspace_selection();
                self.status_message = format!(
                    "Opened workspace {} ({} files).",
                    normalized_root.display(),
                    self.workspace_files.len()
                );
            }
            Err(error) => {
                self.workspace_files.clear();
                self.workspace_selected = None;
                self.workspace_expanded_folders.clear();
                self.status_message = format!(
                    "Workspace scan failed for {}: {error}",
                    normalized_root.display()
                );
            }
        }

        self.workspace_ui_dirty = true;
    }

    fn open_workspace_file(&mut self, index: usize) {
        let Some(entry) = self.workspace_files.get(index) else {
            self.status_message = "Workspace file selection is out of range.".to_string();
            return;
        };

        self.load_from_path(entry.path.clone());
    }

    fn toggle_workspace_folder(&mut self, folder_key: &str) {
        if self.workspace_expanded_folders.contains(folder_key) {
            self.workspace_expanded_folders.remove(folder_key);
        } else {
            self.workspace_expanded_folders
                .insert(folder_key.to_owned());
        }
        self.workspace_ui_dirty = true;
    }

    fn sync_workspace_selection(&mut self) {
        self.workspace_selected = self
            .workspace_files
            .iter()
            .position(|entry| entry.path == self.paths.load_path);
        self.workspace_ui_dirty = true;
    }

    fn reparse(&mut self) {
        self.parsed = parse_document_with_format(&self.document, self.document_format);
        self.mark_processed_cache_dirty_from(0);
    }

    fn reparse_with_dirty_hint(&mut self, dirty_line: usize) {
        self.parsed = parse_document_with_format(&self.document, self.document_format);
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

    fn max_top_line(&self, _visible_lines: usize) -> usize {
        self.document.line_count().saturating_sub(1)
    }

    fn clamp_scroll(&mut self, visible_lines: usize) {
        let max_top = self.max_top_line(visible_lines);
        self.top_line = self.top_line.min(max_top);
    }

    fn clamp_processed_top_line(&mut self) {
        let max_top = self.document.line_count().saturating_sub(1);
        self.processed_top_line = self.processed_top_line.min(max_top);
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
        self.processed_top_line = self.top_line;
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
                self.status_message = format!("Saved {}", status_path_label(&path));
            }
            Err(error) => {
                self.status_message =
                    format!("Save failed for {}: {error}", status_path_label(&path));
            }
        }
    }

    fn load_from_path(&mut self, path: PathBuf) {
        match Document::load(&path) {
            Ok(document) => {
                let document_format = detect_document_format(&path, &document);
                self.document = document;
                self.document_format = document_format;
                self.reparse();
                self.cursor = Cursor::default();
                self.top_line = 0;
                self.processed_top_line = 0;
                self.processed_top_visual = 0;
                self.plain_horizontal_scroll = 0.0;
                self.processed_horizontal_scroll = 0.0;
                self.processed_zoom_anchor_bias_px = 0.0;
                self.clear_history();
                self.paths.load_path = path.clone();
                self.paths.save_path = path.clone();
                self.status_message = format!(
                    "Loaded {} ({}).",
                    status_path_label(&path),
                    document_format_label(self.document_format)
                );
                self.sync_workspace_selection();
                self.reset_blink();
            }
            Err(error) => {
                self.status_message = format!("Load failed for {}: {error}", status_path_label(&path));
            }
        }
    }

    fn history_snapshot(&self) -> EditorHistorySnapshot {
        EditorHistorySnapshot {
            document: self.document.clone(),
            cursor: self.cursor,
            top_line: self.top_line,
            processed_top_line: self.processed_top_line,
            processed_top_visual: self.processed_top_visual,
            plain_horizontal_scroll: self.plain_horizontal_scroll,
            processed_horizontal_scroll: self.processed_horizontal_scroll,
            processed_zoom_anchor_bias_px: self.processed_zoom_anchor_bias_px,
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
        self.parsed = parse_document_with_format(&self.document, self.document_format);
        self.processed_cache = None;
        self.processed_cache_dirty_from_line = Some(0);

        self.cursor = snapshot.cursor;
        self.cursor.position = self.document.clamp_position(self.cursor.position);
        self.cursor.preferred_column = self
            .cursor
            .preferred_column
            .min(self.document.line_len_chars(self.cursor.position.line));

        self.top_line = snapshot.top_line;
        self.processed_top_line = snapshot.processed_top_line;
        self.processed_top_visual = snapshot.processed_top_visual;
        self.plain_horizontal_scroll = snapshot.plain_horizontal_scroll;
        self.processed_horizontal_scroll = snapshot.processed_horizontal_scroll;
        self.processed_zoom_anchor_bias_px = snapshot.processed_zoom_anchor_bias_px;
        self.clamp_scroll(visible_lines);
        self.clamp_processed_top_line();
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

#[derive(Clone, Debug)]
struct WorkspaceFileEntry {
    path: PathBuf,
    relative_display: String,
}

#[derive(Clone, Debug)]
enum WorkspaceSidebarRow {
    Folder {
        folder_key: String,
        folder_name: String,
        depth: usize,
        expanded: bool,
    },
    File {
        file_index: usize,
        file_name: String,
        depth: usize,
    },
}

fn workspace_sidebar_rows(state: &EditorState) -> Vec<WorkspaceSidebarRow> {
    let mut folders_by_parent = BTreeMap::<String, Vec<(String, String)>>::new();
    let mut files_by_parent = BTreeMap::<String, Vec<(usize, String)>>::new();

    for (index, file) in state.workspace_files.iter().enumerate() {
        let parent_key = workspace_parent_key(&file.relative_display);
        let file_name = workspace_base_name(&file.relative_display);
        files_by_parent
            .entry(parent_key)
            .or_default()
            .push((index, file_name));

        let components = file.relative_display.split('/').collect::<Vec<_>>();
        if components.len() <= 1 {
            continue;
        }

        let mut parent = String::new();
        for component in components.iter().take(components.len().saturating_sub(1)) {
            let folder_key = if parent.is_empty() {
                (*component).to_owned()
            } else {
                format!("{parent}/{component}")
            };

            let siblings = folders_by_parent.entry(parent.clone()).or_default();
            if !siblings
                .iter()
                .any(|(existing_key, _)| *existing_key == folder_key)
            {
                siblings.push((folder_key.clone(), (*component).to_owned()));
            }

            parent = folder_key;
        }
    }

    for folders in folders_by_parent.values_mut() {
        folders.sort_by(|left, right| left.1.cmp(&right.1));
    }
    for files in files_by_parent.values_mut() {
        files.sort_by(|left, right| left.1.cmp(&right.1));
    }

    let mut rows = Vec::<WorkspaceSidebarRow>::new();
    append_workspace_sidebar_rows(
        "",
        0,
        &state.workspace_expanded_folders,
        &folders_by_parent,
        &files_by_parent,
        &mut rows,
    );
    rows
}

fn append_workspace_sidebar_rows(
    parent_key: &str,
    depth: usize,
    expanded_folders: &BTreeSet<String>,
    folders_by_parent: &BTreeMap<String, Vec<(String, String)>>,
    files_by_parent: &BTreeMap<String, Vec<(usize, String)>>,
    out: &mut Vec<WorkspaceSidebarRow>,
) {
    if let Some(folders) = folders_by_parent.get(parent_key) {
        for (folder_key, folder_name) in folders {
            let expanded = expanded_folders.contains(folder_key);
            out.push(WorkspaceSidebarRow::Folder {
                folder_key: folder_key.clone(),
                folder_name: folder_name.clone(),
                depth,
                expanded,
            });
            if expanded {
                append_workspace_sidebar_rows(
                    folder_key,
                    depth.saturating_add(1),
                    expanded_folders,
                    folders_by_parent,
                    files_by_parent,
                    out,
                );
            }
        }
    }

    if let Some(files) = files_by_parent.get(parent_key) {
        for (file_index, file_name) in files {
            out.push(WorkspaceSidebarRow::File {
                file_index: *file_index,
                file_name: file_name.clone(),
                depth,
            });
        }
    }
}

fn workspace_parent_key(relative_display: &str) -> String {
    relative_display
        .rsplit_once('/')
        .map_or_else(String::new, |(parent, _)| parent.to_owned())
}

fn workspace_base_name(relative_display: &str) -> String {
    relative_display
        .rsplit('/')
        .next()
        .map_or_else(String::new, str::to_owned)
}

fn default_expanded_workspace_folders(files: &[WorkspaceFileEntry]) -> BTreeSet<String> {
    let mut expanded = BTreeSet::<String>::new();
    for file in files {
        let Some((top_level, _)) = file.relative_display.split_once('/') else {
            continue;
        };
        expanded.insert(top_level.to_owned());
    }
    expanded
}

fn document_format_label(format: DocumentFormat) -> &'static str {
    match format {
        DocumentFormat::Fountain => "Fountain",
        DocumentFormat::Markdown => "Markdown",
    }
}

fn detect_document_format(path: &Path, document: &Document) -> DocumentFormat {
    let path_format = DocumentFormat::from_path(path);
    if path_format == DocumentFormat::Markdown {
        return DocumentFormat::Markdown;
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    if matches!(extension.as_deref(), Some("fountain")) {
        return DocumentFormat::Fountain;
    }

    if looks_like_markdown_document(document) {
        DocumentFormat::Markdown
    } else {
        path_format
    }
}

fn looks_like_markdown_document(document: &Document) -> bool {
    let mut markdown_hits = 0usize;
    let mut fountain_hits = 0usize;

    for line in document.lines().iter().take(300) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if is_markdown_hint(trimmed) {
            markdown_hits += 1;
        }
        if is_fountain_hint(trimmed) {
            fountain_hits += 1;
        }

        if markdown_hits >= 3 && markdown_hits >= fountain_hits.saturating_add(1) {
            return true;
        }
    }

    markdown_hits >= 2 && markdown_hits > fountain_hits
}

fn is_markdown_hint(trimmed: &str) -> bool {
    if trimmed.starts_with('#')
        || trimmed.starts_with('>')
        || trimmed.starts_with("```")
        || trimmed.starts_with("~~~")
        || trimmed.starts_with('|')
    {
        return true;
    }

    if is_markdown_bullet_hint(trimmed) || is_markdown_ordered_list_hint(trimmed) {
        return true;
    }

    let compact = trimmed.replace([' ', '\t'], "");
    let compact_bytes = compact.as_bytes();
    compact_bytes.len() >= 3
        && (compact_bytes.iter().all(|byte| *byte == b'-')
            || compact_bytes.iter().all(|byte| *byte == b'*')
            || compact_bytes.iter().all(|byte| *byte == b'_'))
}

fn is_markdown_bullet_hint(trimmed: &str) -> bool {
    let mut chars = trimmed.chars();
    let Some(marker) = chars.next() else {
        return false;
    };
    if !matches!(marker, '-' | '*' | '+') {
        return false;
    }
    chars.next().is_some_and(char::is_whitespace)
}

fn is_markdown_ordered_list_hint(trimmed: &str) -> bool {
    let mut digits = 0usize;
    for ch in trimmed.chars() {
        if ch.is_ascii_digit() {
            digits += 1;
        } else {
            break;
        }
    }
    if digits == 0 {
        return false;
    }

    let mut chars = trimmed.chars().skip(digits);
    if chars.next() != Some('.') {
        return false;
    }
    chars.next().is_some_and(char::is_whitespace)
}

fn is_fountain_hint(trimmed: &str) -> bool {
    let upper = trimmed.to_ascii_uppercase();
    let is_scene_heading = ["INT.", "EXT.", "EST.", "INT/EXT.", "I/E."]
        .iter()
        .any(|prefix| upper.starts_with(prefix));
    if is_scene_heading {
        return true;
    }

    if upper.ends_with(" TO:")
        || upper == "CUT TO:"
        || upper == "FADE OUT."
        || upper == "FADE TO BLACK."
    {
        return true;
    }

    if trimmed.chars().count() > 32 {
        return false;
    }
    let words = trimmed.split_whitespace().count();
    if words == 0 || words > 4 || trimmed.ends_with(':') {
        return false;
    }

    trimmed
        .chars()
        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || " .()'-".contains(ch))
}

fn collect_workspace_files(root: &Path) -> io::Result<Vec<WorkspaceFileEntry>> {
    let mut files = Vec::<WorkspaceFileEntry>::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                if should_skip_workspace_dir(&path) {
                    continue;
                }
                stack.push(path);
                continue;
            }

            if !file_type.is_file() || !is_workspace_file_candidate(&path) {
                continue;
            }

            let relative = path
                .strip_prefix(root)
                .unwrap_or(path.as_path())
                .to_string_lossy()
                .replace('\\', "/");

            files.push(WorkspaceFileEntry {
                path,
                relative_display: relative,
            });
        }
    }

    files.sort_by(|left, right| left.relative_display.cmp(&right.relative_display));
    Ok(files)
}

fn should_skip_workspace_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    name.starts_with('.') || matches!(name, "target" | "node_modules")
}

fn is_workspace_file_candidate(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    matches!(
        extension.as_deref(),
        Some("fountain") | Some("txt") | Some("md") | Some("markdown")
    )
}
