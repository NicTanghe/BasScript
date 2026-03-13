use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use basscript_core::{
    Cursor, Document, DocumentFormat, DocumentPath, LineKind, LinkDisplayText, ParsedLine,
    Position, ScriptLink, parse_document_with_format,
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
const EDITOR_SETTINGS_PATH: &str = "settings/editor_settings.ron";
const KEYBINDS_SETTINGS_PATH: &str = "settings/keybinds.ron";
const UI_STATE_PATH: &str = "settings/state.ron";
const THEME_SETTINGS_PATH: &str = "settings/theme.ron";
const LEGACY_EDITOR_SETTINGS_PATH: &str = "scripts/editor_settings.ron";
const LEGACY_KEYBINDS_SETTINGS_PATH: &str = "scripts/keybinds.ron";
const LEGACY_SETTINGS_PATH: &str = "scripts/settings.toml";
const PROCESSED_PAPER_CAPACITY: usize = 16;
const SELECTION_RECT_CAPACITY: usize = 512;

const FONT_SIZE: f32 = 12.0;
const LINE_HEIGHT: f32 = 12.0;
const DEFAULT_CHAR_WIDTH: f32 = 7.2;
const DEFAULT_MARKDOWN_CHAR_WIDTH: f32 = 6.2;
const TEXT_PADDING_X: f32 = 14.0;
const TEXT_PADDING_Y: f32 = 10.0;
const ZOOM_MIN: f32 = 0.6;
const ZOOM_MAX: f32 = 1.8;
const ZOOM_STEP: f32 = 0.1;
const NAVIGATION_REPEAT_INITIAL_DELAY_SECS: f32 = 0.30;
const NAVIGATION_REPEAT_INTERVAL_SECS: f32 = 0.045;
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
const THEME_COLOR_WHEEL_SIZE_PX: u32 = 192;
const THEME_COLOR_WHEEL_SIZE: f32 = THEME_COLOR_WHEEL_SIZE_PX as f32;
const THEME_COLOR_SLIDER_WIDTH: f32 = 180.0;
const THEME_COLOR_SLIDER_HEIGHT: f32 = 14.0;
const THEME_COLOR_SLIDER_KNOB_WIDTH: f32 = 8.0;
const LINK_HOVER_HSV_VALUE_STEP: f32 = 0.02;
const LINK_HOVER_HSV_VALUE_MAX: f32 = 0.50;
const PROCESSED_LINE_SPAN_PARTS: usize = 24;
const MIN_TEXT_BOX_WIDTH: f32 = 120.0;
const MIN_TEXT_BOX_HEIGHT: f32 = 120.0;
const PANEL_SPLITTER_WIDTH: f32 = 0.0;
const PANEL_SPLITTER_PICK_RADIUS: f32 = 18.0;
const WORKSPACE_WIDTH_DEFAULT: f32 = 280.0;
const WORKSPACE_WIDTH_MIN: f32 = 180.0;
const EDITOR_PANEL_MIN_WIDTH: f32 = 220.0;
const UNDECORATED_WINDOW_CORNER_RADIUS: f32 = 8.0;

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
const COLOR_WORKSPACE_BG: Color = Color::srgb(0.86, 0.87, 0.89);
const COLOR_PAPER: Color = Color::srgb(1.0, 1.0, 1.0);
const COLOR_TEXT_MAIN: Color = Color::srgb(0.18, 0.19, 0.20);
const COLOR_TEXT_MUTED: Color = Color::srgb(0.34, 0.36, 0.39);
const COLOR_WORKSPACE_FILE: Color = Color::srgb(0.18, 0.19, 0.20);
const COLOR_WORKSPACE_FILE_HOVER: Color = Color::srgb(0.10, 0.35, 0.62);
const COLOR_WORKSPACE_FILE_SELECTED: Color = Color::srgb(0.69, 0.28, 0.22);
const COLOR_SPLITTER_IDLE: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
const COLOR_SPLITTER_HOVER: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
const COLOR_SPLITTER_ACTIVE: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);

pub struct UiPlugin;

#[derive(States, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
enum UiScreenState {
    #[default]
    Editor,
    Settings,
    Keybinds,
    Theme,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorState>()
            .init_resource::<DialogState>()
            .init_resource::<MiddleAutoscrollState>()
            .init_resource::<NavigationRepeatState>()
            .init_resource::<MouseSelectionState>()
            .init_resource::<PanelLayoutState>()
            .init_resource::<PanelSplitterDragState>()
            .init_state::<UiScreenState>()
            .insert_non_send_resource(DialogMainThreadMarker)
            .add_systems(
                Startup,
                (
                    setup,
                    setup_selection_rects.after(setup),
                    setup_processed_papers.after(setup),
                ),
            )
            .add_systems(
                Update,
                (
                    style_toolbar_buttons,
                    style_workspace_file_entry_text,
                    handle_window_shortcuts,
                    sync_window_chrome,
                    sync_glass_surfaces,
                    sync_top_menu_visibility,
                    sync_rounded_window_surfaces,
                    sync_panel_display_mode,
                    sync_panel_split_layout,
                    sync_settings_ui,
                    sync_theme_picker_ui,
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
                    .run_if(
                        in_state(UiScreenState::Settings)
                            .or(in_state(UiScreenState::Keybinds))
                            .or(in_state(UiScreenState::Theme)),
                    ),
            )
            .add_systems(
                Update,
                handle_theme_color_picker_buttons.run_if(in_state(UiScreenState::Theme)),
            )
            .add_systems(
                Update,
                handle_theme_color_picker_input.run_if(in_state(UiScreenState::Theme)),
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
                    handle_panel_splitter_drag.after(handle_middle_mouse_autoscroll),
                    handle_mouse_selection
                        .after(handle_middle_mouse_autoscroll)
                        .after(handle_panel_splitter_drag),
                    sync_hovered_processed_link
                        .after(handle_mouse_selection)
                        .before(render_editor),
                    sync_middle_autoscroll_indicator.after(handle_middle_mouse_autoscroll),
                    style_panel_splitters,
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
struct EditorBodyRow;

#[derive(Component)]
struct WorkspaceSidebarPane;

#[derive(Component)]
struct EditorPanelsContainer;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct PanelPaneSlot {
    kind: PanelKind,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum PanelSplitter {
    Workspace,
    Panels,
}

#[derive(Component)]
struct PanelText {
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
struct PanelSelectionLayer {
    kind: PanelKind,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct PanelSelectionRect {
    kind: PanelKind,
    index: usize,
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
    part_index: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct ProcessedChecklistIcon {
    slot: usize,
    line_offset: usize,
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
    ToggleProcessedGlass,
    ToggleExplorerGlass,
    ToggleTopMenuGlass,
    MarginLeftDecrease,
    MarginLeftIncrease,
    MarginRightDecrease,
    MarginRightIncrease,
    MarginTopDecrease,
    MarginTopIncrease,
    MarginBottomDecrease,
    MarginBottomIncrease,
    LinkHoverHsvValueDecrease,
    LinkHoverHsvValueIncrease,
    OpenTheme,
    OpenLinkColors,
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
    ToggleExplorer,
    ToggleTopMenu,
}

const SHORTCUT_ACTIONS: [ShortcutAction; 11] = [
    ShortcutAction::OpenWorkspace,
    ShortcutAction::SaveAs,
    ShortcutAction::Undo,
    ShortcutAction::Redo,
    ShortcutAction::ZoomIn,
    ShortcutAction::ZoomOut,
    ShortcutAction::PlainView,
    ShortcutAction::ProcessedView,
    ShortcutAction::ProcessedRawCurrentLineView,
    ShortcutAction::ToggleExplorer,
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
    toggle_explorer: ShortcutBinding,
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
            toggle_explorer: ShortcutBinding {
                key: KeyCode::KeyE,
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
            ShortcutAction::ToggleExplorer => self.toggle_explorer,
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
            ShortcutAction::ToggleExplorer => self.toggle_explorer = binding,
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
        ShortcutAction::ToggleExplorer => "Toggle Explorer",
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
        ShortcutAction::ToggleExplorer => "Toggle explorer",
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
        ShortcutAction::ToggleExplorer => "toggle_explorer",
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

#[derive(Component)]
struct WindowSurfaceRoot;

#[derive(Component)]
struct StatusLineRoot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct SettingToggleLabel {
    action: SettingsAction,
}

#[derive(Component)]
struct SettingsScreenRoot;

#[derive(Component)]
struct KeybindsScreenRoot;

#[derive(Component)]
struct ThemeScreenRoot;

#[derive(Component)]
struct TopMenuSection;

#[derive(Component)]
struct ThemeOnlySettingControl;

fn window_surface_border_radius(show_system_titlebar: bool) -> BorderRadius {
    let radius = if show_system_titlebar {
        0.0
    } else {
        UNDECORATED_WINDOW_CORNER_RADIUS
    };
    BorderRadius::all(px(radius))
}

fn window_surface_overflow(show_system_titlebar: bool) -> Overflow {
    if show_system_titlebar {
        Overflow::visible()
    } else {
        Overflow::clip()
    }
}

fn window_surface_top_border_radius(round_left: bool, round_right: bool) -> BorderRadius {
    let radius = px(UNDECORATED_WINDOW_CORNER_RADIUS);
    BorderRadius::new(
        if round_left { radius } else { px(0.0) },
        if round_right { radius } else { px(0.0) },
        px(0.0),
        px(0.0),
    )
}

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

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum ThemeColorChannel {
    Hue,
    Saturation,
    Red,
    Green,
    Blue,
    Value,
    Alpha,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct ThemeColorRow {
    target: ThemeColorTarget,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct ThemeColorLabel {
    channel: ThemeColorChannel,
}

#[derive(Component)]
struct ThemeScreenTitleLabel;

#[derive(Component)]
struct ThemeScreenDescriptionLabel;

#[derive(Component)]
struct ThemeColorNameLabel {
    target: ThemeColorTarget,
}

#[derive(Component)]
struct ThemeColorValueLabel {
    target: ThemeColorTarget,
}

#[derive(Component)]
struct ThemeColorPickerPanel;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct ThemeColorPreviewSwatch {
    target: ThemeColorTarget,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct ThemeColorPickerButton {
    target: ThemeColorTarget,
}

#[derive(Component)]
struct ThemeLinkHoverSettingRow;

#[derive(Component)]
struct ThemeLinkHoverValueLabel;

#[derive(Component)]
struct ThemeHueSatWheel;

#[derive(Component)]
struct ThemeHueSatCursor;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum ThemeSliderChannel {
    Hue,
    Saturation,
    Red,
    Green,
    Blue,
    Value,
    Alpha,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThemeColorTarget {
    SelectionBackground,
    LinkFallback,
    LinkProp,
    LinkPlace,
    LinkCharacter,
    LinkFaction,
    LinkConcept,
}

impl ThemeColorTarget {
    fn screen_title(self) -> &'static str {
        if self.is_link_color() {
            "Link Colors"
        } else {
            "Theme"
        }
    }

    fn screen_description(self) -> &'static str {
        if self.is_link_color() {
            "Adjust processed-view link colors by YAML `type`. Unmapped types use Fallback, and hover uses the HSV value offset."
        } else {
            "Adjust editor selection colors and glass surfaces."
        }
    }

    fn color_label(self) -> &'static str {
        match self {
            Self::SelectionBackground => "Selection background",
            Self::LinkFallback => "Fallback",
            Self::LinkProp => "Prop",
            Self::LinkPlace => "Place",
            Self::LinkCharacter => "Character",
            Self::LinkFaction => "Faction",
            Self::LinkConcept => "Concept",
        }
    }

    fn status_label(self) -> &'static str {
        match self {
            Self::SelectionBackground => "selection background",
            Self::LinkFallback => "fallback link color",
            Self::LinkProp => "prop link color",
            Self::LinkPlace => "place link color",
            Self::LinkCharacter => "character link color",
            Self::LinkFaction => "faction link color",
            Self::LinkConcept => "concept link color",
        }
    }

    fn is_link_color(self) -> bool {
        !matches!(self, Self::SelectionBackground)
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct ThemeColorSlider {
    channel: ThemeSliderChannel,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct ThemeColorSliderKnob {
    channel: ThemeSliderChannel,
}

#[derive(Component)]
struct ThemeSelectionHsvLabel;

#[derive(Component)]
struct ThemeSelectionRgbLabel;

#[derive(Component)]
struct ThemeSelectionHexLabel;

#[derive(Clone, Debug, PartialEq, Eq)]
struct HoveredProcessedLink {
    source_line: usize,
    raw_start_column: usize,
    raw_end_column: usize,
}

#[derive(Resource)]
struct EditorState {
    document: Document,
    parsed: Vec<ParsedLine>,
    document_format: DocumentFormat,
    cursor: Cursor,
    selection_anchor: Option<Position>,
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
    workspace_sidebar_visible: bool,
    top_menu_collapsed: bool,
    processed_glass: bool,
    explorer_glass: bool,
    top_menu_glass: bool,
    selection_bg_rgba: Vec4,
    selection_bg_color: Color,
    link_fallback_rgba: Vec4,
    link_fallback_color: Color,
    link_prop_rgba: Vec4,
    link_prop_color: Color,
    link_place_rgba: Vec4,
    link_place_color: Color,
    link_character_rgba: Vec4,
    link_character_color: Color,
    link_faction_rgba: Vec4,
    link_faction_color: Color,
    link_concept_rgba: Vec4,
    link_concept_color: Color,
    link_hover_hsv_value_adjustment: f32,
    theme_color_target: ThemeColorTarget,
    theme_color_picker_open: bool,
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
    script_link_target_types: BTreeMap<String, String>,
    missing_script_link_targets: BTreeSet<String>,
    hovered_processed_link: Option<HoveredProcessedLink>,
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

#[derive(Resource, Default, Clone, Copy, Debug)]
struct NavigationRepeatState {
    active_arrow: Option<KeyCode>,
    repeat_cooldown_secs: f32,
}

#[derive(Resource, Clone, Copy, Debug)]
struct PanelLayoutState {
    workspace_width_px: f32,
    plain_ratio: f32,
}

impl Default for PanelLayoutState {
    fn default() -> Self {
        Self {
            workspace_width_px: WORKSPACE_WIDTH_DEFAULT,
            plain_ratio: 0.5,
        }
    }
}

#[derive(Resource, Default, Clone, Copy, Debug)]
struct PanelSplitterDragState {
    active: Option<PanelSplitter>,
    last_cursor_x: Option<f32>,
    suppress_next_left_click: bool,
}

enum PendingDialog {
    Workspace(Task<Option<PathBuf>>),
    Save(Task<Option<PathBuf>>),
}

struct DialogMainThreadMarker;

#[derive(Clone, Debug)]
struct PersistentSettings {
    dialogue_double_space_newline: bool,
    non_dialogue_double_space_newline: bool,
    show_system_titlebar: bool,
    page_margin_left: f32,
    page_margin_right: f32,
    page_margin_top: f32,
    page_margin_bottom: f32,
    workspace_root_path: Option<String>,
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
            workspace_root_path: None,
        }
    }
}

#[derive(Clone, Debug)]
struct PersistentUiState {
    workspace_sidebar_visible: bool,
    top_menu_collapsed: bool,
}

impl Default for PersistentUiState {
    fn default() -> Self {
        Self {
            workspace_sidebar_visible: true,
            top_menu_collapsed: false,
        }
    }
}

#[derive(Clone, Debug)]
struct ThemeSettings {
    selection_background: Vec4,
    link_fallback: Vec4,
    link_prop: Vec4,
    link_place: Vec4,
    link_character: Vec4,
    link_faction: Vec4,
    link_concept: Vec4,
    link_hover_hsv_value_adjustment: f32,
    processed_glass: bool,
    explorer_glass: bool,
    top_menu_glass: bool,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            selection_background: Vec4::new(0.16, 0.43, 0.88, 0.36),
            link_fallback: Vec4::new(0.10, 0.38, 0.72, 1.0),
            link_prop: Vec4::new(0.68, 0.40, 0.10, 1.0),
            link_place: Vec4::new(0.12, 0.50, 0.34, 1.0),
            link_character: Vec4::new(0.70, 0.20, 0.24, 1.0),
            link_faction: Vec4::new(0.34, 0.32, 0.68, 1.0),
            link_concept: Vec4::new(0.56, 0.28, 0.14, 1.0),
            link_hover_hsv_value_adjustment: 0.10,
            processed_glass: false,
            explorer_glass: false,
            top_menu_glass: false,
        }
    }
}

impl ThemeSettings {
    fn selection_background_clamped(&self) -> Vec4 {
        Vec4::new(
            self.selection_background.x.clamp(0.0, 1.0),
            self.selection_background.y.clamp(0.0, 1.0),
            self.selection_background.z.clamp(0.0, 1.0),
            self.selection_background.w.clamp(0.0, 1.0),
        )
    }

    fn selection_background_color(&self) -> Color {
        let rgba = self.selection_background_clamped();
        Color::srgba(
            rgba.x,
            rgba.y,
            rgba.z,
            rgba.w,
        )
    }

    fn link_fallback_clamped(&self) -> Vec4 {
        Vec4::new(
            self.link_fallback.x.clamp(0.0, 1.0),
            self.link_fallback.y.clamp(0.0, 1.0),
            self.link_fallback.z.clamp(0.0, 1.0),
            self.link_fallback.w.clamp(0.0, 1.0),
        )
    }

    fn link_fallback_color(&self) -> Color {
        let rgba = self.link_fallback_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    fn link_prop_clamped(&self) -> Vec4 {
        Vec4::new(
            self.link_prop.x.clamp(0.0, 1.0),
            self.link_prop.y.clamp(0.0, 1.0),
            self.link_prop.z.clamp(0.0, 1.0),
            self.link_prop.w.clamp(0.0, 1.0),
        )
    }

    fn link_prop_color(&self) -> Color {
        let rgba = self.link_prop_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    fn link_place_clamped(&self) -> Vec4 {
        Vec4::new(
            self.link_place.x.clamp(0.0, 1.0),
            self.link_place.y.clamp(0.0, 1.0),
            self.link_place.z.clamp(0.0, 1.0),
            self.link_place.w.clamp(0.0, 1.0),
        )
    }

    fn link_place_color(&self) -> Color {
        let rgba = self.link_place_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    fn link_character_clamped(&self) -> Vec4 {
        Vec4::new(
            self.link_character.x.clamp(0.0, 1.0),
            self.link_character.y.clamp(0.0, 1.0),
            self.link_character.z.clamp(0.0, 1.0),
            self.link_character.w.clamp(0.0, 1.0),
        )
    }

    fn link_character_color(&self) -> Color {
        let rgba = self.link_character_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    fn link_faction_clamped(&self) -> Vec4 {
        Vec4::new(
            self.link_faction.x.clamp(0.0, 1.0),
            self.link_faction.y.clamp(0.0, 1.0),
            self.link_faction.z.clamp(0.0, 1.0),
            self.link_faction.w.clamp(0.0, 1.0),
        )
    }

    fn link_faction_color(&self) -> Color {
        let rgba = self.link_faction_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    fn link_concept_clamped(&self) -> Vec4 {
        Vec4::new(
            self.link_concept.x.clamp(0.0, 1.0),
            self.link_concept.y.clamp(0.0, 1.0),
            self.link_concept.z.clamp(0.0, 1.0),
            self.link_concept.w.clamp(0.0, 1.0),
        )
    }

    fn link_concept_color(&self) -> Color {
        let rgba = self.link_concept_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    fn link_hover_hsv_value_adjustment_clamped(&self) -> f32 {
        self.link_hover_hsv_value_adjustment
            .clamp(0.0, LINK_HOVER_HSV_VALUE_MAX)
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
struct ChecklistIcons {
    unchecked: Handle<Image>,
    checked: Handle<Image>,
}

#[derive(Resource, Clone)]
struct ThemePickerAssets {
    hue_sat_wheel: Handle<Image>,
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
        let ui_state = load_persistent_ui_state();
        let theme_settings = load_theme_settings();
        let saved_workspace_root = settings.workspace_root_path.clone();
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
            selection_anchor: None,
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
            workspace_sidebar_visible: ui_state.workspace_sidebar_visible,
            top_menu_collapsed: ui_state.top_menu_collapsed,
            processed_glass: theme_settings.processed_glass,
            explorer_glass: theme_settings.explorer_glass,
            top_menu_glass: theme_settings.top_menu_glass,
            selection_bg_rgba: theme_settings.selection_background_clamped(),
            selection_bg_color: theme_settings.selection_background_color(),
            link_fallback_rgba: theme_settings.link_fallback_clamped(),
            link_fallback_color: theme_settings.link_fallback_color(),
            link_prop_rgba: theme_settings.link_prop_clamped(),
            link_prop_color: theme_settings.link_prop_color(),
            link_place_rgba: theme_settings.link_place_clamped(),
            link_place_color: theme_settings.link_place_color(),
            link_character_rgba: theme_settings.link_character_clamped(),
            link_character_color: theme_settings.link_character_color(),
            link_faction_rgba: theme_settings.link_faction_clamped(),
            link_faction_color: theme_settings.link_faction_color(),
            link_concept_rgba: theme_settings.link_concept_clamped(),
            link_concept_color: theme_settings.link_concept_color(),
            link_hover_hsv_value_adjustment: theme_settings
                .link_hover_hsv_value_adjustment_clamped(),
            theme_color_target: ThemeColorTarget::SelectionBackground,
            theme_color_picker_open: false,
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
            script_link_target_types: BTreeMap::new(),
            missing_script_link_targets: BTreeSet::new(),
            hovered_processed_link: None,
            workspace_ui_dirty: true,
            undo_history: Vec::new(),
            redo_history: Vec::new(),
        };
        normalize_page_margins(&mut next);
        let initial_status = next.status_message.clone();
        apply_initial_workspace_root(&mut next, &initial_status, saved_workspace_root.as_deref());
        next
    }
}

impl EditorState {
    fn any_glass_enabled(&self) -> bool {
        self.processed_glass || self.explorer_glass || self.top_menu_glass
    }

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

    fn reparse(&mut self) {
        self.parsed = parse_document_with_format(&self.document, self.document_format);
        self.missing_script_link_targets.clear();
        self.mark_processed_cache_dirty_from(0);
    }

    fn reparse_with_dirty_hint(&mut self, dirty_line: usize) {
        self.parsed = parse_document_with_format(&self.document, self.document_format);
        self.missing_script_link_targets.clear();
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

    fn selection_bounds(&self) -> Option<(Position, Position)> {
        let anchor = self.selection_anchor?;
        let head = self.cursor.position;
        if anchor == head {
            return None;
        }

        if position_is_before_or_equal(anchor, head) {
            Some((anchor, head))
        } else {
            Some((head, anchor))
        }
    }

    fn delete_selection(&mut self) -> Option<Position> {
        let (start, end) = self.selection_bounds()?;
        let next = self.document.delete_range(start, end);
        self.set_cursor(next, true);
        Some(next)
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
        self.set_cursor_with_selection(position, update_preferred, false);
    }

    fn set_cursor_with_selection(
        &mut self,
        position: Position,
        update_preferred: bool,
        extend_selection: bool,
    ) {
        let anchor = if extend_selection {
            Some(self.selection_anchor.unwrap_or(self.cursor.position))
        } else {
            None
        };
        let clamped = self.document.clamp_position(position);

        if update_preferred {
            self.cursor.set_position(clamped);
        } else {
            self.cursor.position = clamped;
        }

        self.selection_anchor = anchor;
        if self.selection_anchor.is_some_and(|start| start == self.cursor.position) {
            self.selection_anchor = None;
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
                self.clear_script_link_target_cache();
                self.reparse();
                self.cursor = Cursor::default();
                self.selection_anchor = None;
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
        self.selection_anchor = None;

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

fn document_format_label(format: DocumentFormat) -> &'static str {
    match format {
        DocumentFormat::Fountain => "Fountain",
        DocumentFormat::Markdown => "Markdown",
    }
}

fn position_is_before_or_equal(left: Position, right: Position) -> bool {
    left.line < right.line || (left.line == right.line && left.column <= right.column)
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

