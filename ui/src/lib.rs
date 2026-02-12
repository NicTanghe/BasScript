use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{Duration, Instant},
};

use basscript_core::{Cursor, Document, DocumentPath, ParsedLine, Position, parse_document};
use bevy::{
    log::{info, warn},
    input::{
        keyboard::{Key, KeyboardInput},
        mouse::{MouseScrollUnit, MouseWheel},
    },
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures_lite::future},
    text::{LineHeight, TextLayoutInfo},
    ui::RelativeCursorPosition,
    window::{PrimaryWindow, RawHandleWrapper},
};
use rfd::AsyncFileDialog;

const FONT_PATH: &str = "fonts/Courier Prime/Courier Prime.ttf";
const DEFAULT_LOAD_PATH: &str = "docs/humanDOC.md";
const DEFAULT_SAVE_PATH: &str = "scripts/session.fountain";

const FONT_SIZE: f32 = 20.0;
const LINE_HEIGHT: f32 = 24.0;
const DEFAULT_CHAR_WIDTH: f32 = 12.0;
const TEXT_PADDING_X: f32 = 14.0;
const TEXT_PADDING_Y: f32 = 10.0;
const CARET_WIDTH: f32 = 2.0;
const CARET_X_OFFSET: f32 = -1.0;
const BUTTON_NORMAL: Color = Color::srgb(0.20, 0.24, 0.29);
const BUTTON_HOVER: Color = Color::srgb(0.28, 0.33, 0.39);
const BUTTON_PRESSED: Color = Color::srgb(0.35, 0.43, 0.50);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorState>()
            .init_resource::<DialogState>()
            .insert_non_send_resource(DialogMainThreadMarker)
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    handle_toolbar_buttons,
                    style_toolbar_buttons,
                    handle_file_shortcuts,
                    resolve_dialog_results,
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
struct StatusText;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum ToolbarAction {
    Load,
    SaveAs,
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

    let font = asset_server.load(FONT_PATH);

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
        }
    }
}

fn style_toolbar_buttons(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<ToolbarAction>),
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
        info!("[dialog] Load dialog preferred directory: {}", directory.display());
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
        let result = request.await.map(|file_handle| file_handle.path().to_path_buf());
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
        info!("[dialog] Save dialog preferred directory: {}", directory.display());
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
        let result = request.await.map(|file_handle| file_handle.path().to_path_buf());
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
    let should_log_watchdog = dialogs
        .last_watchdog_log_at
        .map_or(true, |last| now.duration_since(last) >= Duration::from_secs(2));
    if should_log_watchdog {
        if let Some(opened_at) = dialogs.opened_at {
            let elapsed_ms = opened_at.elapsed().as_millis();
            info!(
                "[dialog] {} dialog pending for {}ms (poll_count={})",
                pending_kind,
                elapsed_ms,
                dialogs.poll_count
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
        pending_kind,
        elapsed_ms,
        dialogs.poll_count
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

    let visible_lines = viewport_lines(&body_query);
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
    let visible_lines = viewport_lines(&body_query);
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
    let visible_lines = viewport_lines(&body_query);
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
    let visible_lines = viewport_lines_from_panels(&panel_query);
    let plain_lines = visible_plain_lines(&state, visible_lines);
    let processed_lines = visible_processed_lines(&state, visible_lines);
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

        let line_offset = panel_layout
            .and_then(|layout| {
                line_index_from_layout_y(layout, local_y, visible_lines, inverse_scale)
            })
            .unwrap_or_else(|| {
                ((local_y / LINE_HEIGHT).floor().max(0.0) as usize)
                    .min(visible_lines.saturating_sub(1))
            });

        let line = state
            .top_line
            .saturating_add(line_offset)
            .min(state.document.line_count().saturating_sub(1));
        let visible_offset = line.saturating_sub(state.top_line);
        let display_line = match panel.kind {
            PanelKind::Plain => plain_lines
                .get(visible_offset)
                .map_or("", |line| line.as_str()),
            PanelKind::Processed => processed_lines
                .get(visible_offset)
                .map_or("", |line| line.as_str()),
        };

        let display_column = match panel.kind {
            PanelKind::Plain => plain_layout
                .and_then(|layout| {
                    column_from_layout_x(
                        layout,
                        visible_offset,
                        local_x,
                        display_line,
                        inverse_scale,
                    )
                })
                .unwrap_or_else(|| (local_x / DEFAULT_CHAR_WIDTH).round().max(0.0) as usize),
            PanelKind::Processed => processed_layout
                .and_then(|layout| {
                    column_from_layout_x(
                        layout,
                        visible_offset,
                        local_x,
                        display_line,
                        inverse_scale,
                    )
                })
                .unwrap_or_else(|| (local_x / DEFAULT_CHAR_WIDTH).round().max(0.0) as usize),
        };

        let raw_column = match panel.kind {
            PanelKind::Plain => display_column,
            PanelKind::Processed => {
                let indent = state
                    .parsed
                    .get(line)
                    .map_or(0, basscript_core::ParsedLine::indent_width);
                display_column.saturating_sub(indent)
            }
        };

        let max_col = state.document.line_len_chars(line);
        let column = raw_column.min(max_col);

        state.set_cursor(Position { line, column }, true);
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
    text_layout_query: Query<(&PanelText, &TextLayoutInfo)>,
    mut caret_query: Query<(&PanelCaret, &mut Node, &mut Visibility)>,
    mut status_query: Query<&mut Text, (With<StatusText>, Without<PanelText>, Without<PanelCaret>)>,
    mut state: ResMut<EditorState>,
) {
    let visible_lines = viewport_lines(&body_query);
    let inverse_scale = body_query
        .iter()
        .next()
        .map(ComputedNode::inverse_scale_factor)
        .unwrap_or(1.0);
    state.clamp_scroll(visible_lines);

    let plain_lines = visible_plain_lines(&state, visible_lines);
    let processed_lines = visible_processed_lines(&state, visible_lines);
    let plain_view = plain_lines.join("\n");
    let processed_view = processed_lines.join("\n");

    for (panel_text, mut text) in text_query.iter_mut() {
        **text = match panel_text.kind {
            PanelKind::Plain => plain_view.clone(),
            PanelKind::Processed => processed_view.clone(),
        };
    }

    if let Ok(mut status) = status_query.single_mut() {
        **status = state.visible_status();
    }

    let plain_layout = panel_layout_info(&text_layout_query, PanelKind::Plain);
    let processed_layout = panel_layout_info(&text_layout_query, PanelKind::Processed);

    let in_view = state.cursor.position.line >= state.top_line
        && state.cursor.position.line < state.top_line + visible_lines;

    for (panel_caret, mut node, mut visibility) in caret_query.iter_mut() {
        if !state.caret_visible || !in_view {
            *visibility = Visibility::Hidden;
            continue;
        }

        let line_offset = state.cursor.position.line - state.top_line;

        let display_column = match panel_caret.kind {
            PanelKind::Plain => state.cursor.position.column,
            PanelKind::Processed => state
                .parsed
                .get(state.cursor.position.line)
                .map_or(state.cursor.position.column, |line| {
                    line.processed_column(state.cursor.position.column)
                }),
        };

        let line_text = match panel_caret.kind {
            PanelKind::Plain => plain_lines
                .get(line_offset)
                .map_or("", |line| line.as_str()),
            PanelKind::Processed => processed_lines
                .get(line_offset)
                .map_or("", |line| line.as_str()),
        };
        let clamped_display_column = display_column.min(line_text.chars().count());
        let byte_index = char_to_byte_index(line_text, clamped_display_column);

        let panel_layout = match panel_caret.kind {
            PanelKind::Plain => plain_layout,
            PanelKind::Processed => processed_layout,
        };
        let caret_x = panel_layout
            .and_then(|layout| {
                caret_x_from_layout(layout, line_offset, line_text, byte_index, inverse_scale)
            })
            .unwrap_or(clamped_display_column as f32 * DEFAULT_CHAR_WIDTH);
        let caret_top = panel_layout
            .and_then(|layout| line_top_from_layout(layout, line_offset, inverse_scale))
            .unwrap_or(line_offset as f32 * LINE_HEIGHT);

        node.left = px(TEXT_PADDING_X + (caret_x + CARET_X_OFFSET).max(0.0));
        node.top = px(TEXT_PADDING_Y + caret_top.max(0.0));
        node.width = px(CARET_WIDTH);
        node.height = px(LINE_HEIGHT);
        *visibility = Visibility::Visible;
    }
}

fn viewport_lines(body_query: &Query<&ComputedNode, With<PanelBody>>) -> usize {
    let Some(computed) = body_query.iter().next() else {
        return 24;
    };

    let logical_height = computed.size().y * computed.inverse_scale_factor();
    let usable_height = (logical_height - (TEXT_PADDING_Y * 2.0)).max(LINE_HEIGHT);
    (usable_height / LINE_HEIGHT).floor().max(1.0) as usize
}

fn viewport_lines_from_panels(
    panel_query: &Query<(&PanelBody, &RelativeCursorPosition, &ComputedNode)>,
) -> usize {
    let Some((_, _, computed)) = panel_query.iter().next() else {
        return 24;
    };

    let logical_height = computed.size().y * computed.inverse_scale_factor();
    let usable_height = (logical_height - (TEXT_PADDING_Y * 2.0)).max(LINE_HEIGHT);
    (usable_height / LINE_HEIGHT).floor().max(1.0) as usize
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

fn visible_processed_lines(state: &EditorState, visible_lines: usize) -> Vec<String> {
    let last = state
        .top_line
        .saturating_add(visible_lines)
        .min(state.parsed.len());

    state
        .parsed
        .iter()
        .skip(state.top_line)
        .take(last.saturating_sub(state.top_line))
        .map(ParsedLine::processed_text)
        .collect()
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

fn layout_line_centers(layout: &TextLayoutInfo, inverse_scale: f32) -> Vec<(usize, f32)> {
    let mut per_line = BTreeMap::<usize, (f32, usize)>::new();

    for glyph in &layout.glyphs {
        let entry = per_line.entry(glyph.line_index).or_insert((0.0, 0));
        entry.0 += glyph.position.y * inverse_scale;
        entry.1 += 1;
    }

    per_line
        .into_iter()
        .filter_map(|(line_index, (sum_y, count))| {
            (count > 0).then_some((line_index, sum_y / count as f32))
        })
        .collect()
}

fn fit_line_centers(samples: &[(usize, f32)]) -> Option<(f32, f32)> {
    if samples.is_empty() {
        return None;
    }

    if samples.len() == 1 {
        let x = samples[0].0 as f32;
        let y = samples[0].1;
        return Some((y - x * LINE_HEIGHT, LINE_HEIGHT));
    }

    let n = samples.len() as f32;
    let mean_x = samples.iter().map(|(x, _)| *x as f32).sum::<f32>() / n;
    let mean_y = samples.iter().map(|(_, y)| *y).sum::<f32>() / n;

    let (numerator, denominator) = samples.iter().fold((0.0_f32, 0.0_f32), |acc, (x, y)| {
        let dx = *x as f32 - mean_x;
        let dy = *y - mean_y;
        (acc.0 + dx * dy, acc.1 + dx * dx)
    });

    let mut slope = if denominator > f32::EPSILON {
        numerator / denominator
    } else {
        LINE_HEIGHT
    };

    if !slope.is_finite() || slope < 0.1 {
        slope = LINE_HEIGHT;
    }

    let intercept = mean_y - slope * mean_x;
    Some((intercept, slope))
}

fn line_center_from_layout(
    layout: &TextLayoutInfo,
    line_index: usize,
    inverse_scale: f32,
) -> Option<f32> {
    let samples = layout_line_centers(layout, inverse_scale);
    let (intercept, slope) = fit_line_centers(&samples)?;
    Some(intercept + slope * line_index as f32)
}

fn line_top_from_layout(
    layout: &TextLayoutInfo,
    line_index: usize,
    inverse_scale: f32,
) -> Option<f32> {
    line_center_from_layout(layout, line_index, inverse_scale)
        .map(|center| center - LINE_HEIGHT * 0.5)
}

fn line_index_from_layout_y(
    layout: &TextLayoutInfo,
    y: f32,
    visible_lines: usize,
    inverse_scale: f32,
) -> Option<usize> {
    let samples = layout_line_centers(layout, inverse_scale);
    let (intercept, slope) = fit_line_centers(&samples)?;

    let mut best_line = 0usize;
    let mut best_distance = f32::MAX;

    for line in 0..visible_lines.max(1) {
        let center_y = intercept + slope * line as f32;
        let distance = (center_y - y).abs();
        if distance < best_distance {
            best_distance = distance;
            best_line = line;
        }
    }

    Some(best_line)
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
