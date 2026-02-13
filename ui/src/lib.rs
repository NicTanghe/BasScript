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
    ui::RelativeCursorPosition,
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
const PROCESSED_SPAN_CAPACITY: usize = 256;

const FONT_SIZE: f32 = 20.0;
const LINE_HEIGHT: f32 = 24.0;
const DEFAULT_CHAR_WIDTH: f32 = 12.0;
const TEXT_PADDING_X: f32 = 14.0;
const TEXT_PADDING_Y: f32 = 10.0;
const CARET_WIDTH: f32 = 2.0;
const CARET_X_OFFSET: f32 = -1.0;
const CARET_Y_OFFSET_FACTOR: f32 = -0.12;
const BUTTON_NORMAL: Color = Color::srgb(0.20, 0.24, 0.29);
const BUTTON_HOVER: Color = Color::srgb(0.28, 0.33, 0.39);
const BUTTON_PRESSED: Color = Color::srgb(0.35, 0.43, 0.50);
const COLOR_ACTION: Color = Color::srgb(0.93, 0.93, 0.93);
const COLOR_SCENE: Color = Color::srgb(0.98, 0.97, 0.90);
const COLOR_CHARACTER: Color = Color::srgb(0.95, 0.92, 0.78);
const COLOR_DIALOGUE: Color = Color::srgb(0.94, 0.94, 0.94);
const COLOR_PARENTHETICAL: Color = Color::srgb(0.72, 0.78, 0.84);
const COLOR_TRANSITION: Color = Color::srgb(0.82, 0.90, 0.98);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorState>()
            .init_resource::<DialogState>()
            .insert_non_send_resource(DialogMainThreadMarker)
            .add_systems(Startup, (setup, setup_processed_spans.after(setup)))
            .add_systems(
                Update,
                (
                    handle_toolbar_buttons,
                    style_toolbar_buttons,
                    handle_settings_buttons,
                    handle_file_shortcuts,
                    resolve_dialog_results,
                    sync_settings_ui,
                    handle_text_input,
                    handle_navigation_input,
                    handle_mouse_scroll,
                    handle_mouse_click,
                    blink_caret,
                    render_editor,
                ),
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
struct ProcessedLineSpan {
    line_offset: usize,
}

#[derive(Component)]
struct StatusText;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum ToolbarAction {
    Load,
    SaveAs,
    Settings,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsAction {
    DialogueDoubleSpaceNewline,
}

#[derive(Component)]
struct SettingsPanel;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct SettingToggleLabel {
    action: SettingsAction,
}

#[derive(Resource)]
struct EditorState {
    document: Document,
    parsed: Vec<ParsedLine>,
    cursor: Cursor,
    top_line: usize,
    paths: DocumentPath,
    status_message: String,
    caret_blink: Timer,
    caret_visible: bool,
    settings_open: bool,
    dialogue_double_space_newline: bool,
    measured_line_step: f32,
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

#[derive(Clone, Copy, Debug, Default)]
struct PersistentSettings {
    dialogue_double_space_newline: bool,
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

        Self {
            document,
            parsed,
            cursor: Cursor::default(),
            top_line: 0,
            paths,
            status_message,
            caret_blink: Timer::from_seconds(0.5, TimerMode::Repeating),
            caret_visible: true,
            settings_open: false,
            dialogue_double_space_newline: settings.dialogue_double_space_newline,
            measured_line_step: LINE_HEIGHT,
        }
    }
}

impl EditorState {
    fn reparse(&mut self) {
        self.parsed = parse_document(&self.document);
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
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, IsDefaultUiCamera));

    let fonts = EditorFonts {
        regular: asset_server.load(FONT_PATH),
        bold: asset_server.load(FONT_BOLD_PATH),
        italic: asset_server.load(FONT_ITALIC_PATH),
        bold_italic: asset_server.load(FONT_BOLD_ITALIC_PATH),
    };
    let font = fonts.regular.clone();
    commands.insert_resource(fonts);

    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.07, 0.08, 0.09)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(px(12.0), px(8.0)),
                    ..default()
                },
                children![
                    (
                        Text::new(
                            "Cmd/Ctrl+O load | Cmd/Ctrl+S save | arrows/home/end/page keys move cursor | mouse wheel scroll | click to place cursor",
                        ),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.78, 0.80, 0.84)),
                    ),
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(8.0),
                            ..default()
                        },
                        children![
                            toolbar_button(font.clone(), "Load", ToolbarAction::Load),
                            toolbar_button(font.clone(), "Save As", ToolbarAction::SaveAs),
                            toolbar_button(font.clone(), "Settings", ToolbarAction::Settings),
                        ],
                    )
                ],
            ));

            root.spawn((
                Node {
                    width: percent(100.0),
                    padding: UiRect::axes(px(12.0), px(0.0)),
                    ..default()
                },
                Text::new("Load opens a native file picker. Save As opens a native save dialog."),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.62, 0.67, 0.73)),
            ));

            root.spawn((
                Node {
                    width: percent(100.0),
                    display: Display::None,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(10.0),
                    padding: UiRect::axes(px(12.0), px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.10, 0.11, 0.13)),
                SettingsPanel,
                children![
                    (
                        Text::new("Settings"),
                        TextFont {
                            font: font.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.90, 0.90, 0.92)),
                    ),
                    settings_toggle_button(
                        font.clone(),
                        SettingsAction::DialogueDoubleSpaceNewline,
                    ),
                ],
            ));

            root.spawn((
                Node {
                    width: percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    column_gap: px(10.0),
                    padding: UiRect::axes(px(10.0), px(8.0)),
                    ..default()
                },
                children![
                    panel_bundle(font.clone(), PanelKind::Plain, "Plain"),
                    panel_bundle(font.clone(), PanelKind::Processed, "Processed"),
                ],
            ));

            root.spawn((
                Node {
                    width: percent(100.0),
                    padding: UiRect::axes(px(12.0), px(8.0)),
                    ..default()
                },
                Text::new(""),
                TextFont {
                    font,
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.92, 0.92)),
                StatusText,
            ));
        });
}

fn setup_processed_spans(
    mut commands: Commands,
    fonts: Res<EditorFonts>,
    text_query: Query<(Entity, &PanelText, Option<&Children>)>,
) {
    for (entity, panel_text, children) in text_query.iter() {
        if panel_text.kind != PanelKind::Processed {
            continue;
        }

        if children.is_some_and(|children| !children.is_empty()) {
            continue;
        }

        let regular_font = fonts.regular.clone();

        commands.entity(entity).with_children(|parent| {
            for line_offset in 0..PROCESSED_SPAN_CAPACITY {
                parent.spawn((
                    TextSpan::new(""),
                    TextFont {
                        font: regular_font.clone(),
                        font_size: FONT_SIZE,
                        ..default()
                    },
                    LineHeight::Px(LINE_HEIGHT),
                    TextColor(COLOR_ACTION),
                    ProcessedLineSpan { line_offset },
                ));
            }
        });
    }
}

fn toolbar_button(font: Handle<Font>, label: &str, action: ToolbarAction) -> impl Bundle {
    (
        Button,
        action,
        Node {
            padding: UiRect::axes(px(12.0), px(6.0)),
            ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        children![(
            Text::new(label),
            TextFont {
                font,
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.96, 0.96, 0.96)),
        )],
    )
}

fn settings_toggle_button(font: Handle<Font>, action: SettingsAction) -> impl Bundle {
    (
        Button,
        action,
        Node {
            padding: UiRect::axes(px(12.0), px(6.0)),
            ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        children![(
            Text::new(""),
            TextFont {
                font,
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.96, 0.96, 0.96)),
            SettingToggleLabel { action },
        )],
    )
}

fn panel_bundle(font: Handle<Font>, kind: PanelKind, title: &str) -> impl Bundle {
    (
        Node {
            flex_grow: 1.0,
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.11, 0.12, 0.14)),
        children![
            (
                Node {
                    width: percent(100.0),
                    padding: UiRect::axes(px(14.0), px(6.0)),
                    ..default()
                },
                Text::new(title),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
            ),
            (
                Node {
                    width: percent(100.0),
                    flex_grow: 1.0,
                    position_type: PositionType::Relative,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.09, 0.10, 0.11)),
                RelativeCursorPosition::default(),
                PanelBody { kind },
                children![
                    (
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(TEXT_PADDING_X),
                            top: px(TEXT_PADDING_Y),
                            width: px(CARET_WIDTH),
                            height: px(LINE_HEIGHT),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.95, 0.95, 1.0, 0.32)),
                        Visibility::Hidden,
                        ZIndex(0),
                        PanelCaret { kind },
                    ),
                    (
                        Text::new(""),
                        TextLayout::new_with_no_wrap(),
                        TextFont {
                            font: font.clone(),
                            font_size: FONT_SIZE,
                            ..default()
                        },
                        LineHeight::Px(LINE_HEIGHT),
                        TextColor(Color::srgb(0.93, 0.93, 0.93)),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(TEXT_PADDING_X),
                            top: px(TEXT_PADDING_Y),
                            ..default()
                        },
                        ZIndex(1),
                        PanelText { kind },
                    )
                ],
            )
        ],
    )
}

fn handle_toolbar_buttons(
    _dialog_main_thread: NonSend<DialogMainThreadMarker>,
    interaction_query: Query<(&Interaction, &ToolbarAction), (Changed<Interaction>, With<Button>)>,
    primary_window_query: Query<&RawHandleWrapper, With<PrimaryWindow>>,
    mut state: ResMut<EditorState>,
    mut dialogs: ResMut<DialogState>,
) {
    let parent_handle = primary_window_query.iter().next();

    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        info!(
            "[dialog] Toolbar {:?} pressed (parent_handle: {}, has_pending: {})",
            action,
            parent_handle.is_some(),
            dialogs.pending.is_some()
        );

        match action {
            ToolbarAction::Load => open_load_dialog(&mut state, &mut dialogs, parent_handle),
            ToolbarAction::SaveAs => open_save_dialog(&mut state, &mut dialogs, parent_handle),
            ToolbarAction::Settings => {
                state.settings_open = !state.settings_open;
                state.status_message = if state.settings_open {
                    "Opened settings.".to_string()
                } else {
                    "Closed settings.".to_string()
                };
            }
        }
    }
}

fn style_toolbar_buttons(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<Button>,
            Or<(With<ToolbarAction>, With<SettingsAction>)>,
        ),
    >,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        color.0 = match *interaction {
            Interaction::Pressed => BUTTON_PRESSED,
            Interaction::Hovered => BUTTON_HOVER,
            Interaction::None => BUTTON_NORMAL,
        };
    }
}

fn handle_settings_buttons(
    interaction_query: Query<(&Interaction, &SettingsAction), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<EditorState>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            SettingsAction::DialogueDoubleSpaceNewline => {
                state.dialogue_double_space_newline = !state.dialogue_double_space_newline;
                let persistent = PersistentSettings {
                    dialogue_double_space_newline: state.dialogue_double_space_newline,
                };

                state.status_message = match save_persistent_settings(&persistent) {
                    Ok(()) => format!(
                        "Dialogue double-space newline in processed pane: {} (saved)",
                        if state.dialogue_double_space_newline {
                            "ON"
                        } else {
                            "OFF"
                        }
                    ),
                    Err(error) => format!(
                        "Dialogue double-space newline in processed pane: {} (save failed: {error})",
                        if state.dialogue_double_space_newline {
                            "ON"
                        } else {
                            "OFF"
                        }
                    ),
                };
            }
        }
    }
}

fn sync_settings_ui(
    state: Res<EditorState>,
    mut panel_query: Query<&mut Node, With<SettingsPanel>>,
    mut toggle_label_query: Query<(&SettingToggleLabel, &mut Text)>,
) {
    if let Ok(mut panel_node) = panel_query.single_mut() {
        panel_node.display = if state.settings_open {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (label, mut text) in toggle_label_query.iter_mut() {
        **text = match label.action {
            SettingsAction::DialogueDoubleSpaceNewline => format!(
                "Double space as newline in dialogue (processed pane): {}",
                if state.dialogue_double_space_newline {
                    "ON"
                } else {
                    "OFF"
                }
            ),
        };
    }
}

fn load_persistent_settings() -> PersistentSettings {
    let path = PathBuf::from(SETTINGS_PATH);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            info!(
                "[settings] No settings file found at {}; using defaults",
                path.display()
            );
            return PersistentSettings::default();
        }
        Err(error) => {
            warn!(
                "[settings] Failed reading {}: {}; using defaults",
                path.display(),
                error
            );
            return PersistentSettings::default();
        }
    };

    let value = if let Some(value) = parse_toml_bool(&contents, "dialogue_double_space_newline") {
        value
    } else if let Some(value) = parse_toml_bool(&contents, "parenthetical_double_space_newline") {
        // Backward-compatibility for the short-lived parenthetical key.
        info!(
            "[settings] Loaded legacy parenthetical_double_space_newline key from {}",
            path.display()
        );
        value
    } else {
        warn!(
            "[settings] Could not parse dialogue_double_space_newline in {}; using defaults",
            path.display()
        );
        return PersistentSettings::default();
    };

    info!("[settings] Loaded settings from {}", path.display());
    PersistentSettings {
        dialogue_double_space_newline: value,
    }
}

fn save_persistent_settings(settings: &PersistentSettings) -> io::Result<()> {
    let path = PathBuf::from(SETTINGS_PATH);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = format!(
        "# BasScript settings\n\
         # true: processed pane renders dialogue double spaces as new lines\n\
         dialogue_double_space_newline = {}\n",
        settings.dialogue_double_space_newline
    );

    fs::write(&path, contents)?;
    info!("[settings] Saved settings to {}", path.display());
    Ok(())
}

fn parse_toml_bool(contents: &str, key: &str) -> Option<bool> {
    for line in contents.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((lhs, rhs)) = line.split_once('=') else {
            continue;
        };
        if lhs.trim() != key {
            continue;
        }

        return match rhs.trim() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        };
    }

    None
}

fn handle_file_shortcuts(
    _dialog_main_thread: NonSend<DialogMainThreadMarker>,
    keys: Res<ButtonInput<KeyCode>>,
    primary_window_query: Query<&RawHandleWrapper, With<PrimaryWindow>>,
    mut state: ResMut<EditorState>,
    mut dialogs: ResMut<DialogState>,
) {
    let parent_handle = primary_window_query.iter().next();
    let shortcut_down = keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);
    if !shortcut_down {
        return;
    }

    if keys.just_pressed(KeyCode::KeyO) {
        info!(
            "[dialog] Shortcut Cmd/Ctrl+O detected (parent_handle: {}, has_pending: {})",
            parent_handle.is_some(),
            dialogs.pending.is_some()
        );
        open_load_dialog(&mut state, &mut dialogs, parent_handle);
    }

    if keys.just_pressed(KeyCode::KeyS) {
        info!(
            "[dialog] Shortcut Cmd/Ctrl+S detected (parent_handle: {}, has_pending: {})",
            parent_handle.is_some(),
            dialogs.pending.is_some()
        );
        open_save_dialog(&mut state, &mut dialogs, parent_handle);
    }
}

fn open_load_dialog(
    state: &mut EditorState,
    dialogs: &mut DialogState,
    parent_handle: Option<&RawHandleWrapper>,
) {
    if dialogs.pending.is_some() {
        let pending_kind = dialogs
            .pending
            .as_ref()
            .map_or("unknown", PendingDialog::kind_name);
        warn!(
            "[dialog] Ignoring load request because {} dialog is already pending",
            pending_kind
        );
        state.status_message = "A file dialog is already open.".to_string();
        return;
    }

    info!(
        "[dialog] Starting load dialog request on thread {:?}",
        std::thread::current().id()
    );

    let mut dialog = AsyncFileDialog::new()
        .set_title("Open Script File")
        .add_filter("Script files", &["fountain", "txt", "md"]);

    if let Some(directory) = preferred_dialog_directory(state) {
        info!(
            "[dialog] Load dialog preferred directory: {}",
            directory.display()
        );
        dialog = dialog.set_directory(directory);
    } else {
        warn!("[dialog] No preferred directory found for load dialog");
    }

    dialog = attach_dialog_parent(dialog, parent_handle);

    info!("[dialog] Creating native load dialog future");
    let request = dialog.pick_file();
    info!("[dialog] Native load future created; spawning task");

    let task = AsyncComputeTaskPool::get().spawn(async move {
        info!("[dialog] Load task awaiting picker result...");
        let result = request
            .await
            .map(|file_handle| file_handle.path().to_path_buf());
        match &result {
            Some(path) => info!("[dialog] Load task received path: {}", path.display()),
            None => info!("[dialog] Load task returned: canceled"),
        }
        result
    });

    dialogs.begin_pending(PendingDialog::Load(task));
    info!("[dialog] Load dialog task spawned");
    state.status_message = "Opening file picker...".to_string();
}

fn open_save_dialog(
    state: &mut EditorState,
    dialogs: &mut DialogState,
    parent_handle: Option<&RawHandleWrapper>,
) {
    if dialogs.pending.is_some() {
        let pending_kind = dialogs
            .pending
            .as_ref()
            .map_or("unknown", PendingDialog::kind_name);
        warn!(
            "[dialog] Ignoring save request because {} dialog is already pending",
            pending_kind
        );
        state.status_message = "A file dialog is already open.".to_string();
        return;
    }

    info!(
        "[dialog] Starting save dialog request on thread {:?}",
        std::thread::current().id()
    );

    let mut dialog = AsyncFileDialog::new()
        .set_title("Save Script File")
        .add_filter("Script files", &["fountain", "txt", "md"]);

    if let Some(directory) = preferred_dialog_directory(state) {
        info!(
            "[dialog] Save dialog preferred directory: {}",
            directory.display()
        );
        dialog = dialog.set_directory(directory);
    } else {
        warn!("[dialog] No preferred directory found for save dialog");
    }

    let default_name = state
        .paths
        .save_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("script.fountain")
        .to_string();

    info!("[dialog] Save dialog default filename: {}", default_name);
    dialog = dialog.set_file_name(default_name.as_str());
    dialog = attach_dialog_parent(dialog, parent_handle);

    info!("[dialog] Creating native save dialog future");
    let request = dialog.save_file();
    info!("[dialog] Native save future created; spawning task");

    let task = AsyncComputeTaskPool::get().spawn(async move {
        info!("[dialog] Save task awaiting picker result...");
        let result = request
            .await
            .map(|file_handle| file_handle.path().to_path_buf());
        match &result {
            Some(path) => info!("[dialog] Save task received path: {}", path.display()),
            None => info!("[dialog] Save task returned: canceled"),
        }
        result
    });

    dialogs.begin_pending(PendingDialog::Save(task));
    info!("[dialog] Save dialog task spawned");
    state.status_message = "Opening save dialog...".to_string();
}

fn attach_dialog_parent(
    dialog: AsyncFileDialog,
    parent_handle: Option<&RawHandleWrapper>,
) -> AsyncFileDialog {
    let Some(parent_handle) = parent_handle else {
        warn!("[dialog] No primary window handle found; opening unparented dialog");
        return dialog;
    };

    // SAFETY: This is called from Bevy update systems on the main app thread.
    let handle = unsafe { parent_handle.get_handle() };
    info!("[dialog] Attached dialog parent to primary window handle");
    dialog.set_parent(&handle)
}

fn resolve_dialog_results(mut state: ResMut<EditorState>, mut dialogs: ResMut<DialogState>) {
    let Some(pending) = dialogs.pending.as_mut() else {
        return;
    };
    let pending_kind = pending.kind_name();

    enum DialogResult {
        Load(Option<PathBuf>),
        Save(Option<PathBuf>),
    }

    let finished = match pending {
        PendingDialog::Load(task) => {
            future::block_on(future::poll_once(task)).map(DialogResult::Load)
        }
        PendingDialog::Save(task) => {
            future::block_on(future::poll_once(task)).map(DialogResult::Save)
        }
    };

    dialogs.poll_count = dialogs.poll_count.saturating_add(1);

    let now = Instant::now();
    let should_log_watchdog = dialogs.last_watchdog_log_at.map_or(true, |last| {
        now.duration_since(last) >= Duration::from_secs(2)
    });
    if should_log_watchdog {
        if let Some(opened_at) = dialogs.opened_at {
            let elapsed_ms = opened_at.elapsed().as_millis();
            info!(
                "[dialog] {} dialog pending for {}ms (poll_count={})",
                pending_kind, elapsed_ms, dialogs.poll_count
            );
        }
        dialogs.last_watchdog_log_at = Some(now);
    }

    let Some(result) = finished else {
        return;
    };

    let elapsed_ms = dialogs
        .opened_at
        .map_or(0_u128, |opened_at| opened_at.elapsed().as_millis());
    info!(
        "[dialog] {} dialog future resolved after {}ms (poll_count={})",
        pending_kind, elapsed_ms, dialogs.poll_count
    );

    dialogs.clear_pending();

    match result {
        DialogResult::Load(Some(path)) => {
            info!("[dialog] Loading selected path: {}", path.display());
            state.load_from_path(path);
        }
        DialogResult::Load(None) => {
            info!("[dialog] Load dialog canceled by user");
            state.status_message = "Load canceled.".to_string();
        }
        DialogResult::Save(Some(path)) => {
            info!("[dialog] Saving to selected path: {}", path.display());
            state.save_to_path(path);
        }
        DialogResult::Save(None) => {
            info!("[dialog] Save dialog canceled by user");
            state.status_message = "Save canceled.".to_string();
        }
    }
}

fn preferred_dialog_directory(state: &EditorState) -> Option<PathBuf> {
    state
        .paths
        .load_path
        .parent()
        .map(|path| path.to_path_buf())
        .or_else(|| {
            state
                .paths
                .save_path
                .parent()
                .map(|path| path.to_path_buf())
        })
}

fn handle_text_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    body_query: Query<&ComputedNode, With<PanelBody>>,
    mut state: ResMut<EditorState>,
) {
    if keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]) {
        return;
    }

    let visible_lines = viewport_lines(&body_query, state.measured_line_step);
    let mut edited = false;

    for input in keyboard_inputs.read() {
        if !input.state.is_pressed() {
            continue;
        }

        let mut changed = false;

        match &input.logical_key {
            Key::Enter => {
                let cursor_pos = state.cursor.position;
                let next = state.document.insert_newline(cursor_pos);
                state.set_cursor(next, true);
                changed = true;
            }
            Key::Backspace => {
                let cursor_pos = state.cursor.position;
                let next = state.document.backspace(cursor_pos);
                state.set_cursor(next, true);
                changed = true;
            }
            Key::Delete => {
                let cursor_pos = state.cursor.position;
                let next = state.document.delete(cursor_pos);
                state.set_cursor(next, false);
                changed = true;
            }
            _ => {
                if let Some(inserted_text) = &input.text {
                    if inserted_text.chars().all(is_printable_char) {
                        let cursor_pos = state.cursor.position;
                        let next = state.document.insert_text(cursor_pos, inserted_text);
                        state.set_cursor(next, true);
                        changed = true;
                    }
                }
            }
        }

        if changed {
            edited = true;
        }
    }

    if edited {
        state.reparse();
        state.ensure_cursor_visible(visible_lines);
    }
}

fn handle_navigation_input(
    keys: Res<ButtonInput<KeyCode>>,
    body_query: Query<&ComputedNode, With<PanelBody>>,
    mut state: ResMut<EditorState>,
) {
    let visible_lines = viewport_lines(&body_query, state.measured_line_step);
    let mut moved = false;

    if keys.just_pressed(KeyCode::ArrowLeft) {
        let next = state.document.move_left(state.cursor.position);
        state.set_cursor(next, true);
        moved = true;
    }

    if keys.just_pressed(KeyCode::ArrowRight) {
        let next = state.document.move_right(state.cursor.position);
        state.set_cursor(next, true);
        moved = true;
    }

    if keys.just_pressed(KeyCode::ArrowUp) {
        let next = state
            .document
            .move_up(state.cursor.position, state.cursor.preferred_column);
        state.set_cursor(next, false);
        moved = true;
    }

    if keys.just_pressed(KeyCode::ArrowDown) {
        let next = state
            .document
            .move_down(state.cursor.position, state.cursor.preferred_column);
        state.set_cursor(next, false);
        moved = true;
    }

    if keys.just_pressed(KeyCode::Home) {
        let line = state.cursor.position.line;
        state.set_cursor(Position { line, column: 0 }, true);
        moved = true;
    }

    if keys.just_pressed(KeyCode::End) {
        let line = state.cursor.position.line;
        let column = state.document.line_len_chars(line);
        state.set_cursor(Position { line, column }, true);
        moved = true;
    }

    let page_step = visible_lines.saturating_sub(1).max(1);

    if keys.just_pressed(KeyCode::PageUp) {
        let new_line = state.cursor.position.line.saturating_sub(page_step);
        let column = state
            .cursor
            .preferred_column
            .min(state.document.line_len_chars(new_line));

        state.set_cursor(
            Position {
                line: new_line,
                column,
            },
            false,
        );
        moved = true;
    }

    if keys.just_pressed(KeyCode::PageDown) {
        let last_line = state.document.line_count().saturating_sub(1);
        let new_line = state
            .cursor
            .position
            .line
            .saturating_add(page_step)
            .min(last_line);
        let column = state
            .cursor
            .preferred_column
            .min(state.document.line_len_chars(new_line));

        state.set_cursor(
            Position {
                line: new_line,
                column,
            },
            false,
        );
        moved = true;
    }

    if moved {
        state.ensure_cursor_visible(visible_lines);
    }
}

fn handle_mouse_scroll(
    mut mouse_wheels: MessageReader<MouseWheel>,
    body_query: Query<&ComputedNode, With<PanelBody>>,
    mut state: ResMut<EditorState>,
) {
    let visible_lines = viewport_lines(&body_query, state.measured_line_step);
    let mut delta_lines: isize = 0;

    for wheel in mouse_wheels.read() {
        let mut delta = -wheel.y;

        if wheel.unit == MouseScrollUnit::Pixel {
            delta /= LINE_HEIGHT;
        }

        delta_lines += delta.round() as isize;
    }

    if delta_lines != 0 {
        state.scroll_by(delta_lines, visible_lines);
        state.clamp_cursor_to_visible_range(visible_lines);
        state.reset_blink();
    }
}

fn handle_mouse_click(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    panel_query: Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
    text_layout_query: Query<(&PanelText, &TextLayoutInfo)>,
    mut state: ResMut<EditorState>,
) {
    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let visible_lines = viewport_lines_from_panels(&panel_query, state.measured_line_step);
    let plain_lines = visible_plain_lines(&state, visible_lines);
    let processed_view = build_processed_view(&state, visible_lines);
    let plain_layout = panel_layout_info(&text_layout_query, PanelKind::Plain);
    let processed_layout = panel_layout_info(&text_layout_query, PanelKind::Processed);

    for (panel, relative_cursor, computed) in panel_query.iter() {
        if !relative_cursor.cursor_over() {
            continue;
        }

        let Some(normalized) = relative_cursor.normalized else {
            continue;
        };

        if state.document.is_empty() {
            state.set_cursor(Position::default(), true);
            break;
        }

        let inverse_scale = computed.inverse_scale_factor();
        let size = computed.size() * inverse_scale;
        let local_x = (normalized.x * size.x - TEXT_PADDING_X).max(0.0);
        let local_y = (normalized.y * size.y - TEXT_PADDING_Y).max(0.0);

        let panel_layout = match panel.kind {
            PanelKind::Plain => plain_layout,
            PanelKind::Processed => processed_layout,
        };
        let panel_line_count = match panel.kind {
            PanelKind::Plain => plain_lines.len().max(1),
            PanelKind::Processed => processed_view.lines.len().max(1),
        };

        // Anchor Y mapping to measured layout origin while keeping fixed line-height steps.
        let line_offset = panel_layout
            .and_then(|layout| {
                line_index_from_layout_y(layout, local_y, panel_line_count, inverse_scale)
            })
            .unwrap_or_else(|| {
                ((local_y / LINE_HEIGHT).floor().max(0.0) as usize)
                    .min(panel_line_count.saturating_sub(1))
            });

        let (line, raw_column) = match panel.kind {
            PanelKind::Plain => {
                let line = state
                    .top_line
                    .saturating_add(line_offset)
                    .min(state.document.line_count().saturating_sub(1));
                let visible_offset = line.saturating_sub(state.top_line);
                let display_line = plain_lines
                    .get(visible_offset)
                    .map_or("", |line| line.as_str());
                let display_column = plain_layout
                    .and_then(|layout| {
                        column_from_layout_x(
                            layout,
                            visible_offset,
                            local_x,
                            display_line,
                            inverse_scale,
                        )
                    })
                    .unwrap_or_else(|| (local_x / DEFAULT_CHAR_WIDTH).round().max(0.0) as usize);
                (line, display_column)
            }
            PanelKind::Processed => {
                let visual_index = line_offset.min(processed_view.lines.len().saturating_sub(1));
                let Some(visual_line) = processed_view.lines.get(visual_index) else {
                    continue;
                };

                let display_line = visual_line.text.as_str();
                let display_column = processed_layout
                    .and_then(|layout| {
                        column_from_layout_x(
                            layout,
                            visual_index,
                            local_x,
                            display_line,
                            inverse_scale,
                        )
                    })
                    .unwrap_or_else(|| (local_x / DEFAULT_CHAR_WIDTH).round().max(0.0) as usize);

                let raw_column =
                    processed_raw_column_from_display(&state, visual_line, display_column);
                (visual_line.source_line, raw_column)
            }
        };

        let max_col = state.document.line_len_chars(line);
        let column = raw_column.min(max_col);

        state.set_cursor(Position { line, column }, true);
        state.ensure_cursor_visible(visible_lines);
        break;
    }
}

fn blink_caret(time: Res<Time>, mut state: ResMut<EditorState>) {
    if state.caret_blink.tick(time.delta()).just_finished() {
        state.caret_visible = !state.caret_visible;
    }
}

fn render_editor(
    body_query: Query<&ComputedNode, With<PanelBody>>,
    mut text_query: Query<(&PanelText, &mut Text), (Without<StatusText>, Without<PanelCaret>)>,
    mut processed_span_query: Query<(
        &ProcessedLineSpan,
        &mut TextSpan,
        &mut TextFont,
        &mut TextColor,
    )>,
    text_layout_query: Query<(&PanelText, &TextLayoutInfo)>,
    mut caret_query: Query<(&PanelCaret, &mut Node, &mut Visibility)>,
    mut status_query: Query<&mut Text, (With<StatusText>, Without<PanelText>, Without<PanelCaret>)>,
    fonts: Res<EditorFonts>,
    mut state: ResMut<EditorState>,
) {
    let visible_lines = viewport_lines(&body_query, state.measured_line_step);
    let inverse_scale = body_query
        .iter()
        .next()
        .map(ComputedNode::inverse_scale_factor)
        .unwrap_or(1.0);
    state.clamp_scroll(visible_lines);

    let plain_lines = visible_plain_lines(&state, visible_lines);
    let processed_view = build_processed_view(&state, visible_lines);
    let plain_view = plain_lines.join("\n");

    for (panel_text, mut text) in text_query.iter_mut() {
        **text = match panel_text.kind {
            PanelKind::Plain => plain_view.clone(),
            PanelKind::Processed => String::new(),
        };
    }

    apply_processed_styles(&mut processed_span_query, &state, &processed_view, &fonts);

    if let Ok(mut status) = status_query.single_mut() {
        **status = state.visible_status();
    }

    let plain_layout = panel_layout_info(&text_layout_query, PanelKind::Plain);
    let processed_layout = panel_layout_info(&text_layout_query, PanelKind::Processed);
    if let Some(measured_step) = plain_layout
        .and_then(|layout| measured_line_step_from_layout(layout, inverse_scale))
        .or_else(|| {
            processed_layout
                .and_then(|layout| measured_line_step_from_layout(layout, inverse_scale))
        })
    {
        state.measured_line_step = measured_step;
    }

    for (panel_caret, mut node, mut visibility) in caret_query.iter_mut() {
        if !state.caret_visible {
            *visibility = Visibility::Hidden;
            continue;
        }

        let (line_offset, display_column, line_text, panel_layout) = match panel_caret.kind {
            PanelKind::Plain => {
                let in_view = state.cursor.position.line >= state.top_line
                    && state.cursor.position.line < state.top_line + visible_lines;
                if !in_view {
                    *visibility = Visibility::Hidden;
                    continue;
                }

                let line_offset = state.cursor.position.line - state.top_line;
                let line_text = plain_lines
                    .get(line_offset)
                    .map_or("", |line| line.as_str());
                (
                    line_offset,
                    state.cursor.position.column,
                    line_text,
                    plain_layout,
                )
            }
            PanelKind::Processed => {
                let Some((visual_index, display_column, line_text)) =
                    processed_caret_visual(&state, &processed_view)
                else {
                    *visibility = Visibility::Hidden;
                    continue;
                };

                (visual_index, display_column, line_text, processed_layout)
            }
        };

        let clamped_display_column = display_column.min(line_text.chars().count());
        let byte_index = char_to_byte_index(line_text, clamped_display_column);
        let caret_x = panel_layout
            .and_then(|layout| {
                caret_x_from_layout(layout, line_offset, line_text, byte_index, inverse_scale)
            })
            .unwrap_or(clamped_display_column as f32 * DEFAULT_CHAR_WIDTH);
        let caret_top = panel_layout
            .and_then(|layout| {
                caret_top_from_layout(layout, line_offset, byte_index, inverse_scale)
                    .or_else(|| line_top_from_layout(layout, line_offset, inverse_scale))
            })
            .unwrap_or(line_offset as f32 * LINE_HEIGHT);

        node.left = px(TEXT_PADDING_X + (caret_x + CARET_X_OFFSET).max(0.0));
        let caret_y_offset = CARET_Y_OFFSET_FACTOR * LINE_HEIGHT;
        node.top = px(TEXT_PADDING_Y + (caret_top + caret_y_offset).max(0.0));
        node.width = px(CARET_WIDTH);
        node.height = px(LINE_HEIGHT);
        *visibility = Visibility::Visible;
    }
}

fn viewport_lines(body_query: &Query<&ComputedNode, With<PanelBody>>, line_step: f32) -> usize {
    let Some(computed) = body_query.iter().next() else {
        return 24;
    };

    let logical_height = computed.size().y * computed.inverse_scale_factor();
    // Text starts at TEXT_PADDING_Y from the top; don't reserve extra bottom padding.
    let step = line_step.max(1.0);
    let usable_height = (logical_height - TEXT_PADDING_Y).max(step);
    (usable_height / step).floor().max(1.0) as usize
}

fn viewport_lines_from_panels(
    panel_query: &Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
    line_step: f32,
) -> usize {
    let Some((_, _, computed)) = panel_query.iter().next() else {
        return 24;
    };

    let logical_height = computed.size().y * computed.inverse_scale_factor();
    let step = line_step.max(1.0);
    let usable_height = (logical_height - TEXT_PADDING_Y).max(step);
    (usable_height / step).floor().max(1.0) as usize
}

fn visible_plain_lines(state: &EditorState, visible_lines: usize) -> Vec<String> {
    let last = state
        .top_line
        .saturating_add(visible_lines)
        .min(state.document.line_count());

    state
        .document
        .lines()
        .iter()
        .skip(state.top_line)
        .take(last.saturating_sub(state.top_line))
        .cloned()
        .collect()
}

#[derive(Clone, Debug)]
struct ProcessedVisualLine {
    source_line: usize,
    text: String,
    raw_start_column: usize,
    raw_end_column: usize,
}

#[derive(Clone, Debug, Default)]
struct ProcessedView {
    lines: Vec<ProcessedVisualLine>,
}

fn build_processed_view(state: &EditorState, visible_lines: usize) -> ProcessedView {
    let max_visible = visible_lines.min(PROCESSED_SPAN_CAPACITY).max(1);
    let all_lines = build_all_processed_visual_lines(state);
    if all_lines.is_empty() {
        return ProcessedView::default();
    }

    let default_start = first_visual_index_for_source_line(&all_lines, state.top_line).unwrap_or(0);
    let cursor_in_plain_view = state.cursor.position.line >= state.top_line
        && state.cursor.position.line < state.top_line + visible_lines;
    let cursor_line_offset = state.cursor.position.line.saturating_sub(state.top_line);

    let mut start_index = default_start;
    if cursor_in_plain_view {
        if let Some((cursor_visual_index, _, _)) =
            processed_cursor_visual_from_lines(state, &all_lines)
        {
            start_index = cursor_visual_index.saturating_sub(cursor_line_offset);
        }
    }

    let max_start = all_lines.len().saturating_sub(max_visible);
    start_index = start_index.min(max_start);
    let end_index = start_index.saturating_add(max_visible).min(all_lines.len());

    ProcessedView {
        lines: all_lines[start_index..end_index].to_vec(),
    }
}

fn build_all_processed_visual_lines(state: &EditorState) -> Vec<ProcessedVisualLine> {
    let mut lines = Vec::<ProcessedVisualLine>::new();

    for (source_line, parsed_line) in state.parsed.iter().enumerate() {
        if state.dialogue_double_space_newline && parsed_line.kind == LineKind::Dialogue {
            let indent = " ".repeat(parsed_line.indent_width());
            for (raw_start_column, segment) in dialogue_segments(&parsed_line.raw) {
                let segment_len = segment.chars().count();
                lines.push(ProcessedVisualLine {
                    source_line,
                    text: format!("{indent}{segment}"),
                    raw_start_column,
                    raw_end_column: raw_start_column.saturating_add(segment_len),
                });
            }
        } else {
            let raw_len = parsed_line.raw.chars().count();
            lines.push(ProcessedVisualLine {
                source_line,
                text: parsed_line.processed_text(),
                raw_start_column: 0,
                raw_end_column: raw_len,
            });
        }
    }

    lines
}

fn first_visual_index_for_source_line(
    lines: &[ProcessedVisualLine],
    source_line: usize,
) -> Option<usize> {
    lines
        .iter()
        .position(|line| line.source_line >= source_line)
}

fn dialogue_segments(input: &str) -> Vec<(usize, String)> {
    let chars = input.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        return vec![(0, String::new())];
    }

    let mut segments = Vec::<(usize, String)>::new();
    let mut start = 0usize;
    let mut index = 0usize;

    while index + 1 < chars.len() {
        if chars[index] == ' ' && chars[index + 1] == ' ' {
            let segment = chars[start..index].iter().collect::<String>();
            segments.push((start, segment));
            index += 2;
            start = index;
            continue;
        }

        index += 1;
    }

    let tail = chars[start..].iter().collect::<String>();
    segments.push((start, tail));

    segments
}

fn processed_raw_column_from_display(
    state: &EditorState,
    visual_line: &ProcessedVisualLine,
    display_column: usize,
) -> usize {
    let indent = state
        .parsed
        .get(visual_line.source_line)
        .map_or(0, ParsedLine::indent_width);

    let segment_len = visual_line
        .raw_end_column
        .saturating_sub(visual_line.raw_start_column);
    let local_column = display_column.saturating_sub(indent).min(segment_len);
    visual_line.raw_start_column.saturating_add(local_column)
}

fn processed_caret_visual<'a>(
    state: &EditorState,
    processed_view: &'a ProcessedView,
) -> Option<(usize, usize, &'a str)> {
    processed_cursor_visual_from_lines(state, &processed_view.lines)
}

fn processed_cursor_visual_from_lines<'a>(
    state: &EditorState,
    lines: &'a [ProcessedVisualLine],
) -> Option<(usize, usize, &'a str)> {
    let source_line = state.cursor.position.line;
    let raw_column = state.cursor.position.column;
    let indent = state
        .parsed
        .get(source_line)
        .map_or(0, ParsedLine::indent_width);

    let relevant = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.source_line == source_line)
        .collect::<Vec<_>>();

    let (default_index, default_line) = *relevant.last()?;

    for (entry_index, (visual_index, visual_line)) in relevant.iter().enumerate() {
        let next_start = relevant
            .get(entry_index + 1)
            .map(|(_, next_line)| next_line.raw_start_column);
        let segment_len = visual_line
            .raw_end_column
            .saturating_sub(visual_line.raw_start_column);
        let local_column = raw_column
            .saturating_sub(visual_line.raw_start_column)
            .min(segment_len);

        if raw_column <= visual_line.raw_end_column
            || next_start.is_some_and(|start| raw_column < start)
            || entry_index + 1 == relevant.len()
        {
            return Some((
                *visual_index,
                indent.saturating_add(local_column),
                &visual_line.text,
            ));
        }
    }

    let default_len = default_line
        .raw_end_column
        .saturating_sub(default_line.raw_start_column);
    Some((
        default_index,
        indent.saturating_add(default_len),
        &default_line.text,
    ))
}

fn apply_processed_styles(
    processed_span_query: &mut Query<(
        &ProcessedLineSpan,
        &mut TextSpan,
        &mut TextFont,
        &mut TextColor,
    )>,
    state: &EditorState,
    processed_view: &ProcessedView,
    fonts: &EditorFonts,
) {
    let visible_count = processed_view.lines.len().min(PROCESSED_SPAN_CAPACITY);

    for (processed_span, mut text_span, mut text_font, mut text_color) in
        processed_span_query.iter_mut()
    {
        let line_offset = processed_span.line_offset;

        if line_offset >= visible_count {
            **text_span = String::new();
            continue;
        }

        let Some(visual_line) = processed_view.lines.get(line_offset) else {
            **text_span = String::new();
            continue;
        };

        let Some(parsed_line) = state.parsed.get(visual_line.source_line) else {
            **text_span = String::new();
            continue;
        };

        let mut line_text = visual_line.text.clone();
        if line_offset + 1 < visible_count {
            line_text.push('\n');
        }

        **text_span = line_text;

        let (font_variant, color) = style_for_line_kind(&parsed_line.kind);
        text_font.font = font_for_variant(fonts, font_variant);
        text_font.font_size = FONT_SIZE;
        text_color.0 = color;
    }
}

fn style_for_line_kind(kind: &LineKind) -> (FontVariant, Color) {
    match kind {
        LineKind::SceneHeading => (FontVariant::Bold, COLOR_SCENE),
        LineKind::Action => (FontVariant::Regular, COLOR_ACTION),
        LineKind::Character => (FontVariant::Bold, COLOR_CHARACTER),
        LineKind::Dialogue => (FontVariant::Regular, COLOR_DIALOGUE),
        LineKind::Parenthetical => (FontVariant::Italic, COLOR_PARENTHETICAL),
        LineKind::Transition => (FontVariant::BoldItalic, COLOR_TRANSITION),
        LineKind::Empty => (FontVariant::Regular, COLOR_ACTION),
    }
}

fn font_for_variant(fonts: &EditorFonts, variant: FontVariant) -> Handle<Font> {
    match variant {
        FontVariant::Regular => fonts.regular.clone(),
        FontVariant::Bold => fonts.bold.clone(),
        FontVariant::Italic => fonts.italic.clone(),
        FontVariant::BoldItalic => fonts.bold_italic.clone(),
    }
}

fn panel_layout_info<'a>(
    text_layout_query: &'a Query<(&PanelText, &TextLayoutInfo)>,
    kind: PanelKind,
) -> Option<&'a TextLayoutInfo> {
    text_layout_query
        .iter()
        .find(|(panel_text, _)| panel_text.kind == kind)
        .map(|(_, layout)| layout)
}

fn layout_line_bounds(layout: &TextLayoutInfo, inverse_scale: f32) -> Vec<(usize, f32, f32)> {
    let mut per_line = BTreeMap::<usize, (f32, f32)>::new();

    for glyph in &layout.glyphs {
        let top = glyph.position.y * inverse_scale;
        let bottom = (glyph.position.y + glyph.size.y) * inverse_scale;
        let entry = per_line.entry(glyph.line_index).or_insert((top, bottom));
        entry.0 = entry.0.min(top);
        entry.1 = entry.1.max(bottom);
    }

    per_line
        .into_iter()
        .map(|(line_index, (top, bottom))| (line_index, top, bottom))
        .collect()
}

fn median(values: &mut [f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Some(values[values.len().saturating_sub(1) / 2])
}

fn default_line_step(samples: &[(usize, f32)], fallback_height: f32) -> f32 {
    let mut steps = samples
        .windows(2)
        .filter_map(|window| {
            let left = window[0];
            let right = window[1];
            let index_delta = right.0.saturating_sub(left.0);
            if index_delta == 0 {
                return None;
            }

            let step = (right.1 - left.1) / index_delta as f32;
            (step.is_finite() && step.abs() > 0.1).then_some(step)
        })
        .collect::<Vec<_>>();

    median(&mut steps).unwrap_or(fallback_height.max(1.0))
}

fn interpolate_line_value(samples: &[(usize, f32)], line_index: usize, step: f32) -> Option<f32> {
    if samples.is_empty() {
        return None;
    }

    match samples.binary_search_by_key(&line_index, |(index, _)| *index) {
        Ok(position) => Some(samples[position].1),
        Err(insert) if insert > 0 && insert < samples.len() => {
            let (left_index, left_value) = samples[insert - 1];
            let (right_index, right_value) = samples[insert];
            let index_span = right_index.saturating_sub(left_index).max(1);
            let t = line_index.saturating_sub(left_index) as f32 / index_span as f32;
            Some(left_value + (right_value - left_value) * t)
        }
        Err(0) => {
            let (first_index, first_value) = samples[0];
            Some(first_value - step * first_index.saturating_sub(line_index) as f32)
        }
        Err(_) => {
            let (last_index, last_value) = samples[samples.len().saturating_sub(1)];
            Some(last_value + step * line_index.saturating_sub(last_index) as f32)
        }
    }
}

fn line_top_from_layout(
    layout: &TextLayoutInfo,
    line_index: usize,
    inverse_scale: f32,
) -> Option<f32> {
    let bounds = layout_line_bounds(layout, inverse_scale);
    let mut heights = bounds
        .iter()
        .map(|(_, top, bottom)| (bottom - top).max(1.0))
        .collect::<Vec<_>>();
    let fallback_height = median(&mut heights).unwrap_or(LINE_HEIGHT);
    let top_samples = bounds
        .iter()
        .map(|(index, top, _)| (*index, *top))
        .collect::<Vec<_>>();
    let step = default_line_step(&top_samples, fallback_height);

    interpolate_line_value(&top_samples, line_index, step)
}

fn line_index_from_layout_y(
    layout: &TextLayoutInfo,
    y: f32,
    visible_lines: usize,
    inverse_scale: f32,
) -> Option<usize> {
    let bounds = layout_line_bounds(layout, inverse_scale);
    if bounds.is_empty() {
        return None;
    }

    let mut heights = bounds
        .iter()
        .map(|(_, top, bottom)| (bottom - top).max(1.0))
        .collect::<Vec<_>>();
    let fallback_height = median(&mut heights).unwrap_or(LINE_HEIGHT);

    let center_samples = bounds
        .iter()
        .map(|(index, top, bottom)| (*index, (*top + *bottom) * 0.5))
        .collect::<Vec<_>>();
    let center_step = default_line_step(&center_samples, fallback_height);

    let mut best_line = 0usize;
    let mut best_distance = f32::MAX;
    for line in 0..visible_lines.max(1) {
        let Some(center_y) = interpolate_line_value(&center_samples, line, center_step) else {
            continue;
        };

        let distance = (center_y - y).abs();
        if distance < best_distance {
            best_distance = distance;
            best_line = line;
        }
    }

    Some(best_line)
}

fn measured_line_step_from_layout(layout: &TextLayoutInfo, inverse_scale: f32) -> Option<f32> {
    let bounds = layout_line_bounds(layout, inverse_scale);
    if bounds.is_empty() {
        return None;
    }

    let mut heights = bounds
        .iter()
        .map(|(_, top, bottom)| (bottom - top).max(1.0))
        .collect::<Vec<_>>();
    let fallback_height = median(&mut heights).unwrap_or(LINE_HEIGHT);
    let top_samples = bounds
        .iter()
        .map(|(index, top, _)| (*index, *top))
        .collect::<Vec<_>>();

    let step = default_line_step(&top_samples, fallback_height).abs();
    Some(step.max(1.0))
}

fn caret_top_from_layout(
    layout: &TextLayoutInfo,
    line_index: usize,
    byte_index: usize,
    inverse_scale: f32,
) -> Option<f32> {
    let mut line_glyphs = layout
        .glyphs
        .iter()
        .filter(|glyph| glyph.line_index == line_index)
        .collect::<Vec<_>>();
    if line_glyphs.is_empty() {
        return None;
    }

    line_glyphs.sort_by_key(|glyph| (glyph.byte_index, glyph.byte_length));

    line_glyphs
        .iter()
        .min_by(|left, right| {
            byte_distance(byte_index, left.byte_index, left.byte_length).cmp(&byte_distance(
                byte_index,
                right.byte_index,
                right.byte_length,
            ))
        })
        .map(|glyph| glyph.position.y * inverse_scale)
}

fn byte_distance(target: usize, start: usize, len: usize) -> usize {
    let end = start.saturating_add(len);
    if target < start {
        start.saturating_sub(target)
    } else if target > end {
        target.saturating_sub(end)
    } else {
        0
    }
}

fn line_boundaries(
    layout: &TextLayoutInfo,
    line_index: usize,
    line_text: &str,
    inverse_scale: f32,
) -> Vec<(usize, f32)> {
    let line_len = line_text.len();
    let mut glyphs = layout
        .glyphs
        .iter()
        .filter(|glyph| glyph.line_index == line_index)
        .collect::<Vec<_>>();

    if glyphs.is_empty() {
        let mut boundaries = Vec::with_capacity(line_len.saturating_add(1));
        for byte_index in 0..=line_len {
            boundaries.push((byte_index, byte_index as f32 * DEFAULT_CHAR_WIDTH));
        }
        return boundaries;
    }

    glyphs.sort_by_key(|glyph| (glyph.byte_index, glyph.byte_length));
    let mut step_candidates = glyphs
        .windows(2)
        .filter_map(|window| {
            let left = window[0];
            let right = window[1];
            let byte_gap = right.byte_index.saturating_sub(left.byte_index);
            if byte_gap == 0 {
                return None;
            }
            let step = (right.position.x - left.position.x) * inverse_scale / byte_gap as f32;
            (step.is_finite() && step.abs() > 0.1).then_some(step)
        })
        .collect::<Vec<_>>();

    step_candidates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let byte_step = step_candidates
        .get(step_candidates.len().saturating_sub(1) / 2)
        .copied()
        .unwrap_or(DEFAULT_CHAR_WIDTH);

    let mut anchors = BTreeMap::<usize, Vec<f32>>::new();

    for glyph in glyphs {
        let start = glyph.byte_index.min(line_len);
        let end = glyph
            .byte_index
            .saturating_add(glyph.byte_length)
            .min(line_len);
        let span_bytes = end.saturating_sub(start).max(1);
        let half_width = byte_step * span_bytes as f32 * 0.5;
        let center_x = glyph.position.x * inverse_scale;
        let left = center_x - half_width;
        let right = center_x + half_width;

        anchors.entry(start).or_default().push(left);
        anchors.entry(end).or_default().push(right);
    }

    let mut known = anchors
        .into_iter()
        .map(|(byte_index, xs)| {
            let sum = xs.iter().copied().sum::<f32>();
            (byte_index, sum / xs.len() as f32)
        })
        .collect::<Vec<_>>();

    if known.is_empty() {
        let mut boundaries = Vec::with_capacity(line_len.saturating_add(1));
        for byte_index in 0..=line_len {
            boundaries.push((byte_index, byte_index as f32 * DEFAULT_CHAR_WIDTH));
        }
        return boundaries;
    }

    known.sort_by_key(|(byte_index, _)| *byte_index);

    let first = known[0];
    let last = known[known.len().saturating_sub(1)];
    let mut boundaries = Vec::with_capacity(line_len.saturating_add(1));
    let mut segment = 0usize;

    for byte_index in 0..=line_len {
        while segment + 1 < known.len() && known[segment + 1].0 <= byte_index {
            segment += 1;
        }

        let x = if byte_index <= first.0 {
            first.1 - (first.0 - byte_index) as f32 * byte_step
        } else if byte_index >= last.0 {
            last.1 + (byte_index - last.0) as f32 * byte_step
        } else {
            let (left_byte, left_x) = known[segment];
            let (right_byte, right_x) = known[segment + 1];
            let gap = right_byte.saturating_sub(left_byte).max(1);
            let t = byte_index.saturating_sub(left_byte) as f32 / gap as f32;
            left_x + (right_x - left_x) * t
        };

        boundaries.push((byte_index, x));
    }

    boundaries
}

fn caret_x_from_layout(
    layout: &TextLayoutInfo,
    line_index: usize,
    line_text: &str,
    byte_index: usize,
    inverse_scale: f32,
) -> Option<f32> {
    let boundaries = line_boundaries(layout, line_index, line_text, inverse_scale);
    boundaries
        .iter()
        .find(|(byte, _)| *byte >= byte_index)
        .map(|(_, x)| *x)
        .or_else(|| boundaries.last().map(|(_, x)| *x))
}

fn column_from_layout_x(
    layout: &TextLayoutInfo,
    line_index: usize,
    x: f32,
    line_text: &str,
    inverse_scale: f32,
) -> Option<usize> {
    let boundaries = line_boundaries(layout, line_index, line_text, inverse_scale);
    let (best_byte, _) = boundaries.iter().min_by(|(_, ax), (_, bx)| {
        (*ax - x)
            .abs()
            .partial_cmp(&(*bx - x).abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    })?;

    Some(byte_to_char_index(line_text, *best_byte))
}

fn char_to_byte_index(input: &str, column: usize) -> usize {
    if column == 0 {
        return 0;
    }

    input
        .char_indices()
        .map(|(byte, _)| byte)
        .nth(column)
        .unwrap_or(input.len())
}

fn byte_to_char_index(input: &str, byte_index: usize) -> usize {
    if byte_index == 0 {
        return 0;
    }

    input
        .char_indices()
        .take_while(|(byte, _)| *byte < byte_index)
        .count()
}

fn is_printable_char(chr: char) -> bool {
    let private_use = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !private_use && !chr.is_ascii_control()
}
