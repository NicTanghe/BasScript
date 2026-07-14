use super::*;

pub(crate) const FONT_PATH: &str = "fonts/Courier Prime/Courier Prime.ttf";
pub(crate) const FONT_BOLD_PATH: &str = "fonts/Courier Prime/Courier Prime Bold.ttf";
pub(crate) const FONT_ITALIC_PATH: &str = "fonts/Courier Prime/Courier Prime Italic.ttf";
pub(crate) const FONT_BOLD_ITALIC_PATH: &str = "fonts/Courier Prime/Courier Prime Bold Italic.ttf";
pub(crate) const FONT_MARKDOWN_PATH: &str = "fonts/segoe-ui-4/Segoe UI.ttf";
pub(crate) const FONT_MARKDOWN_BOLD_PATH: &str = "fonts/segoe-ui-4/Segoe UI Bold.ttf";
pub(crate) const FONT_MARKDOWN_ITALIC_PATH: &str = "fonts/segoe-ui-4/Segoe UI Italic.ttf";
pub(crate) const FONT_MARKDOWN_BOLD_ITALIC_PATH: &str = "fonts/segoe-ui-4/Segoe UI Bold Italic.ttf";
pub(crate) const DEFAULT_LOAD_PATH: &str = "docs/welcome.md";
pub(crate) const DEFAULT_SAVE_PATH: &str = "scripts/session.fountain";
pub(crate) const EDITOR_SETTINGS_PATH: &str = "settings/editor_settings.ron";
pub(crate) const KEYBINDS_SETTINGS_PATH: &str = "settings/keybinds.ron";
pub(crate) const UI_STATE_PATH: &str = "settings/state.ron";
pub(crate) const THEME_SETTINGS_PATH: &str = "settings/theme.ron";
pub(crate) const STORY_TAXONOMY_SETTINGS_PATH: &str = "settings/story_taxonomy.ron";
pub(crate) const LEGACY_EDITOR_SETTINGS_PATH: &str = "scripts/editor_settings.ron";
pub(crate) const LEGACY_KEYBINDS_SETTINGS_PATH: &str = "scripts/keybinds.ron";
pub(crate) const LEGACY_SETTINGS_PATH: &str = "scripts/settings.toml";
pub(crate) const PROCESSED_PAPER_CAPACITY: usize = 16;
pub(crate) const SELECTION_RECT_CAPACITY: usize = 512;

pub(crate) const FONT_SIZE: f32 = 12.0;
pub(crate) const LINE_HEIGHT: f32 = 12.0;
pub(crate) const DEFAULT_CHAR_WIDTH: f32 = 7.2;
pub(crate) const DEFAULT_MARKDOWN_CHAR_WIDTH: f32 = 6.2;
pub(crate) const TEXT_PADDING_X: f32 = 14.0;
pub(crate) const TEXT_PADDING_Y: f32 = 10.0;
pub(crate) const ZOOM_MIN: f32 = 0.6;
pub(crate) const ZOOM_MAX: f32 = 1.8;
pub(crate) const ZOOM_STEP: f32 = 0.1;
pub(crate) const NAVIGATION_REPEAT_INITIAL_DELAY_SECS: f32 = 0.30;
pub(crate) const NAVIGATION_REPEAT_INTERVAL_SECS: f32 = 0.045;
pub(crate) const WORKSPACE_SELECTION_REPEAT_INITIAL_DELAY_SECS: f32 = 0.10;
pub(crate) const WORKSPACE_SELECTION_REPEAT_INTERVAL_SECS: f32 = 0.045;
pub(crate) const HISTORY_LIMIT: usize = 512;
pub(crate) const DOCUMENT_NAVIGATION_HISTORY_LIMIT: usize = 128;
pub(crate) const MM_PER_INCH: f32 = 25.4;
pub(crate) const POINTS_PER_INCH: f32 = 72.0;
pub(crate) const A4_WIDTH_MM: f32 = 210.0;
pub(crate) const A4_HEIGHT_MM: f32 = 297.0;
pub(crate) const A4_WIDTH_POINTS: f32 = A4_WIDTH_MM / MM_PER_INCH * POINTS_PER_INCH;
pub(crate) const A4_HEIGHT_POINTS: f32 = A4_HEIGHT_MM / MM_PER_INCH * POINTS_PER_INCH;
pub(crate) const PAGE_OUTER_MARGIN: f32 = 14.0;
pub(crate) const PAGE_TEXT_MARGIN_LEFT: f32 = 42.0;
pub(crate) const PAGE_TEXT_MARGIN_RIGHT: f32 = 34.0;
pub(crate) const PAGE_TEXT_MARGIN_TOP: f32 = 30.0;
pub(crate) const PAGE_TEXT_MARGIN_BOTTOM: f32 = 30.0;
pub(crate) const PAGE_GAP: f32 = 24.0;
pub(crate) const PAGE_MARGIN_STEP: f32 = 8.0;
pub(crate) const THEME_COLOR_WHEEL_SIZE_PX: u32 = 192;
pub(crate) const THEME_COLOR_WHEEL_SIZE: f32 = THEME_COLOR_WHEEL_SIZE_PX as f32;
pub(crate) const THEME_COLOR_SLIDER_WIDTH: f32 = 180.0;
pub(crate) const THEME_COLOR_SLIDER_HEIGHT: f32 = 14.0;
pub(crate) const THEME_COLOR_SLIDER_KNOB_WIDTH: f32 = 8.0;
pub(crate) const LINK_HOVER_HSV_VALUE_STEP: f32 = 0.02;
pub(crate) const LINK_HOVER_HSV_VALUE_MAX: f32 = 0.50;
pub(crate) const PROCESSED_LINE_SPAN_PARTS: usize = 24;
pub(crate) const PROCESSED_IMAGE_BLOCK_LINES: usize = 14;
pub(crate) const PROCESSED_IMAGE_BLOCK_GAP: f32 = 4.0;
pub(crate) const MIN_TEXT_BOX_WIDTH: f32 = 120.0;
pub(crate) const MIN_TEXT_BOX_HEIGHT: f32 = 120.0;
pub(crate) const PANEL_SPLITTER_WIDTH: f32 = 0.0;
pub(crate) const PANEL_SPLITTER_PICK_RADIUS: f32 = 18.0;
pub(crate) const WORKSPACE_WIDTH_DEFAULT: f32 = 280.0;
pub(crate) const WORKSPACE_WIDTH_MIN: f32 = 180.0;
pub(crate) const EDITOR_PANEL_MIN_WIDTH: f32 = 220.0;
pub(crate) const UNDECORATED_WINDOW_CORNER_RADIUS: f32 = 8.0;

pub(crate) const BUTTON_NORMAL: Color = Color::srgb(0.80, 0.82, 0.84);
pub(crate) const BUTTON_HOVER: Color = Color::srgb(0.74, 0.77, 0.80);
pub(crate) const BUTTON_PRESSED: Color = Color::srgb(0.68, 0.72, 0.76);
pub(crate) const COLOR_ACTION: Color = Color::srgb(0.12, 0.13, 0.15);
pub(crate) const COLOR_SCENE: Color = Color::srgb(0.10, 0.10, 0.12);
pub(crate) const COLOR_CHARACTER: Color = Color::srgb(0.20, 0.16, 0.12);
pub(crate) const COLOR_DIALOGUE: Color = Color::srgb(0.11, 0.12, 0.13);
pub(crate) const COLOR_PARENTHETICAL: Color = Color::srgb(0.24, 0.28, 0.32);
pub(crate) const COLOR_TRANSITION: Color = Color::srgb(0.15, 0.23, 0.31);
pub(crate) const COLOR_MARKDOWN_HEADING: Color = Color::srgb(0.18, 0.24, 0.40);
pub(crate) const COLOR_MARKDOWN_QUOTE: Color = Color::srgb(0.22, 0.29, 0.26);
pub(crate) const COLOR_MARKDOWN_CODE: Color = Color::srgb(0.29, 0.17, 0.18);
pub(crate) const COLOR_MARKDOWN_RULE: Color = Color::srgb(0.35, 0.35, 0.38);
pub(crate) const COLOR_PANEL_BG: Color = Color::srgb(0.89, 0.90, 0.91);
pub(crate) const COLOR_PANEL_BODY_PLAIN: Color = Color::srgb(0.96, 0.96, 0.97);
pub(crate) const COLOR_PANEL_BODY_PROCESSED: Color = Color::srgb(0.82, 0.83, 0.84);
pub(crate) const COLOR_PAPER: Color = Color::srgb(1.0, 1.0, 1.0);
pub(crate) const COLOR_TEXT_MAIN: Color = Color::srgb(0.18, 0.19, 0.20);
pub(crate) const COLOR_TEXT_MUTED: Color = Color::srgb(0.34, 0.36, 0.39);
pub(crate) const COLOR_WORKSPACE_FILE: Color = Color::srgb(0.18, 0.19, 0.20);
pub(crate) const COLOR_WORKSPACE_FILE_HOVER: Color = Color::srgb(0.10, 0.35, 0.62);
pub(crate) const COLOR_WORKSPACE_FILE_SELECTED: Color = Color::srgb(0.69, 0.28, 0.22);
pub(crate) const COLOR_WORKSPACE_ROW_ACTIVE_BG: Color = Color::srgba(0.69, 0.28, 0.22, 0.15);
pub(crate) const COLOR_WORKSPACE_ROW_SELECTED_BG: Color = Color::srgba(0.10, 0.35, 0.62, 0.16);
pub(crate) const COLOR_WORKSPACE_ROW_SELECTED_ACTIVE_BG: Color =
    Color::srgba(0.69, 0.28, 0.22, 0.24);
pub(crate) const COLOR_WORKSPACE_PROMPT_BACKDROP: Color = Color::srgba(0.0, 0.0, 0.0, 0.28);
pub(crate) const COLOR_WORKSPACE_PROMPT_BG: Color = Color::srgb(0.94, 0.95, 0.96);
pub(crate) const COLOR_WORKSPACE_PROMPT_INPUT_BG: Color = Color::srgb(0.99, 0.99, 1.0);
pub(crate) const COLOR_SPLITTER_IDLE: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
pub(crate) const COLOR_SPLITTER_HOVER: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
pub(crate) const COLOR_SPLITTER_ACTIVE: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
pub(crate) const COLOR_IMAGE_PLACEHOLDER: Color = Color::srgb(0.72, 0.74, 0.77);

pub struct UiPlugin;

#[derive(States, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub(crate) enum UiScreenState {
    #[default]
    Editor,
    Settings,
    Keybinds,
    Theme,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorState>()
            .init_resource::<NativeGlassState>()
            .init_resource::<DialogState>()
            .init_resource::<MiddleAutoscrollState>()
            .init_resource::<NavigationRepeatState>()
            .init_resource::<LinkAutocompleteInputCapture>()
            .init_resource::<DocumentNavigationInputCapture>()
            .init_resource::<WorkspaceSelectionRepeatState>()
            .init_resource::<MouseSelectionState>()
            .init_resource::<CanvasDragState>()
            .init_resource::<PanelLayoutState>()
            .init_resource::<ResolvedPanelWidths>()
            .init_resource::<PanelSplitterDragState>()
            .init_resource::<EditorImageCache>()
            .init_state::<UiScreenState>()
            .insert_non_send(DialogMainThreadMarker)
            .add_systems(
                Startup,
                (
                    setup,
                    setup_selection_rects.after(setup),
                    setup_processed_papers.after(setup),
                    setup_story_query_sheet_result_spans.after(setup),
                ),
            )
            .add_systems(
                Update,
                (
                    style_toolbar_buttons,
                    sync_processed_overlay_toggle_group.after(render_editor),
                    style_workspace_file_entry_text,
                    sync_workspace_prompt_ui,
                    handle_window_shortcuts,
                    sync_window_chrome,
                    sync_glass_surfaces,
                    sync_top_menu_visibility,
                    sync_rounded_window_surfaces,
                    sync_panel_display_mode,
                    sync_panel_split_layout
                        .after(handle_window_shortcuts)
                        .after(handle_panel_splitter_drag),
                    sync_command_menu_ui,
                    sync_story_query_sheet_ui,
                    sync_settings_ui,
                    reset_settings_menu_scroll_on_open,
                    sync_theme_picker_ui,
                    sync_workspace_sidebar,
                    sync_workspace_selected_row_scroll.after(sync_workspace_sidebar),
                ),
            )
            .add_systems(
                Update,
                (
                    handle_toolbar_buttons,
                    handle_processed_link_color_toggle,
                    handle_processed_pagination_toggle,
                    handle_formatting_marks_toggle,
                    handle_workspace_file_buttons,
                    handle_workspace_folder_buttons,
                    handle_markdown_metadata_buttons,
                    handle_story_query_sheet_buttons,
                    handle_story_query_sheet_link_click,
                    handle_story_query_sheet_keyboard,
                )
                    .run_if(in_state(UiScreenState::Editor)),
            )
            .add_systems(
                Update,
                (
                    handle_settings_buttons,
                    handle_settings_screen_navigation.run_if(in_state(UiScreenState::Settings)),
                )
                    .run_if(
                        in_state(UiScreenState::Settings)
                            .or_else(in_state(UiScreenState::Keybinds))
                            .or_else(in_state(UiScreenState::Theme)),
                    ),
            )
            .add_systems(
                Update,
                handle_settings_menu_mouse_scroll.run_if(
                    in_state(UiScreenState::Settings).or_else(in_state(UiScreenState::Keybinds)),
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
                (
                    handle_keybind_buttons,
                    handle_keybind_screen_navigation.before(capture_keybind_input),
                    capture_keybind_input,
                )
                    .run_if(in_state(UiScreenState::Keybinds)),
            )
            .add_systems(
                Update,
                (
                    handle_file_shortcuts,
                    handle_workspace_prompt_input,
                    handle_command_menu_input,
                    handle_command_menu_open_input,
                    handle_workspace_keyboard_input,
                    resolve_dialog_results,
                    handle_vim_input,
                    handle_text_input,
                    handle_navigation_input,
                    handle_mouse_scroll,
                    handle_canvas_drag_input,
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
                    render_editor.after(sync_panel_split_layout),
                )
                    .run_if(in_state(UiScreenState::Editor)),
            );
        app.add_systems(
            Update,
            sync_workspace_link_prompt_folder_options
                .before(sync_workspace_prompt_ui)
                .run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            handle_workspace_link_prompt_buttons
                .after(sync_workspace_link_prompt_folder_options)
                .run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            handle_workspace_link_folder_mouse_scroll.run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            handle_formatting_page_break_click
                .after(handle_mouse_selection)
                .before(render_editor)
                .run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            sync_formatting_mark_overlays
                .after(render_editor)
                .run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            handle_document_navigation_history
                .before(handle_story_query_sheet_keyboard)
                .before(handle_command_menu_input)
                .before(handle_markdown_metadata_input)
                .before(handle_vim_input)
                .before(handle_navigation_input)
                .run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            handle_markdown_metadata_input
                .before(handle_workspace_prompt_input)
                .before(handle_command_menu_input)
                .before(handle_vim_input)
                .before(handle_text_input)
                .before(handle_navigation_input)
                .run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            sync_markdown_metadata_controls_ui
                .after(render_editor)
                .run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            handle_workspace_mouse_scroll
                .before(handle_mouse_scroll)
                .run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            handle_story_query_sheet_mouse_scroll
                .before(handle_mouse_scroll)
                .run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            handle_link_autocomplete_keyboard_input
                .before(handle_vim_input)
                .before(handle_text_input)
                .before(handle_navigation_input)
                .before(handle_canvas_text_edit_input)
                .run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            handle_canvas_text_edit_input
                .after(handle_canvas_drag_input)
                .after(handle_vim_input)
                .run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            sync_link_autocomplete_context
                .after(handle_text_input)
                .after(handle_navigation_input)
                .after(handle_vim_input)
                .after(handle_canvas_text_edit_input)
                .after(resolve_dialog_results)
                .run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            sync_link_autocomplete_ui
                .after(sync_link_autocomplete_context)
                .after(render_editor)
                .run_if(in_state(UiScreenState::Editor)),
        );
        app.add_systems(
            Update,
            (
                render_processed_images.after(render_editor),
                sync_canvas_board.after(render_editor),
                sync_canvas_text_overlays.after(sync_canvas_board),
            )
                .run_if(in_state(UiScreenState::Editor)),
        );
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PanelKind {
    Plain,
    Processed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DisplayMode {
    Split,
    Plain,
    Processed,
    ProcessedRawCurrentLine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessedLinkColorMode {
    Colored,
    Hovered,
    Plain,
}

impl ProcessedLinkColorMode {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Colored => Self::Hovered,
            Self::Hovered => Self::Plain,
            Self::Plain => Self::Colored,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Colored => "colored",
            Self::Hovered => "hovered",
            Self::Plain => "plain",
        }
    }

    pub(crate) fn settings_value(self) -> &'static str {
        self.label()
    }

    pub(crate) fn from_settings_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "colored" => Some(Self::Colored),
            "hovered" => Some(Self::Hovered),
            "plain" => Some(Self::Plain),
            _ => None,
        }
    }
}

impl DisplayMode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            DisplayMode::Split => "Split",
            DisplayMode::Plain => "Plain",
            DisplayMode::Processed => "Processed",
            DisplayMode::ProcessedRawCurrentLine => "Processed + Raw Line",
        }
    }

    pub(crate) fn panel_visible(self, panel: PanelKind) -> bool {
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
pub(crate) struct PanelRoot {
    pub(crate) kind: PanelKind,
}

#[derive(Component)]
pub(crate) struct PanelBody {
    pub(crate) kind: PanelKind,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProcessedOverlayToggleGroup {
    pub(crate) kind: PanelKind,
}

#[derive(Component)]
pub(crate) struct ProcessedLinkColorToggle;

#[derive(Component)]
pub(crate) struct ProcessedPaginationToggle;

#[derive(Component)]
pub(crate) struct FormattingMarksToggle;

#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct ProcessedOverlayToggleSpring {
    pub(crate) offset_x: f32,
    pub(crate) velocity_x: f32,
    pub(crate) velocity_y: f32,
    pub(crate) touching_page: bool,
    pub(crate) initialized: bool,
    pub(crate) phase: ProcessedOverlayTogglePhase,
    pub(crate) compression_distance: f32,
    pub(crate) previous_page_right: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ProcessedOverlayTogglePhase {
    #[default]
    Idle,
    Compressing,
    Rebounding,
    ReturningUnderPage,
}

#[derive(Component)]
pub(crate) struct ProcessedLinkColorToggleLabel;

#[derive(Component)]
pub(crate) struct ProcessedPaginationToggleLabel;

#[derive(Component)]
pub(crate) struct FormattingMarksToggleLabel;

#[derive(Resource, Default)]
pub(crate) struct DocumentNavigationInputCapture {
    pub(crate) captured: bool,
}

#[derive(Component)]
pub(crate) struct EditorBodyRow;

#[derive(Component)]
pub(crate) struct WorkspaceSidebarPane;

#[derive(Component)]
pub(crate) struct EditorPanelsContainer;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PanelPaneSlot {
    pub(crate) kind: PanelKind,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PanelSplitter {
    Workspace,
    Panels,
}

#[derive(Component)]
pub(crate) struct PanelText {
    pub(crate) kind: PanelKind,
}

#[derive(Component)]
pub(crate) struct PanelPaper {
    pub(crate) kind: PanelKind,
    pub(crate) slot: usize,
}

#[derive(Component)]
pub(crate) struct PanelSelectionLayer {
    pub(crate) kind: PanelKind,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PanelSelectionRect {
    pub(crate) kind: PanelKind,
    pub(crate) index: usize,
}

#[derive(Component)]
pub(crate) struct MiddleAutoscrollIndicator;

#[derive(Component)]
pub(crate) struct ProcessedPaperText {
    pub(crate) slot: usize,
    pub(crate) line_offset: usize,
}

#[derive(Component)]
pub(crate) struct ProcessedPaperLineSpan {
    pub(crate) slot: usize,
    pub(crate) line_offset: usize,
    pub(crate) part_index: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProcessedChecklistIcon {
    pub(crate) slot: usize,
    pub(crate) line_offset: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProcessedImageBlockNode {
    pub(crate) slot: usize,
    pub(crate) line_offset: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToolbarAction {
    OpenWorkspace,
    Save,
    SaveAs,
    ExportPdf,
    StoryQuerySheet,
    ZoomOut,
    ZoomIn,
    Settings,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsAction {
    DialogueDoubleSpaceNewline,
    NonDialogueDoubleSpaceNewline,
    ToggleProcessedPagination,
    ToggleVimMode,
    ShowSystemTitlebar,
    ToggleProcessedGlass,
    ToggleExplorerGlass,
    ToggleSettingsGlass,
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
pub(crate) enum ShortcutAction {
    NavigateForward,
    OpenWorkspace,
    Save,
    SaveAs,
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    PlainView,
    ProcessedView,
    ProcessedRawCurrentLineView,
    SplitView,
    ToggleExplorer,
    ToggleTopMenu,
}

pub(crate) const SHORTCUT_ACTIONS: [ShortcutAction; 14] = [
    ShortcutAction::NavigateForward,
    ShortcutAction::OpenWorkspace,
    ShortcutAction::Save,
    ShortcutAction::SaveAs,
    ShortcutAction::Undo,
    ShortcutAction::Redo,
    ShortcutAction::ZoomIn,
    ShortcutAction::ZoomOut,
    ShortcutAction::PlainView,
    ShortcutAction::ProcessedView,
    ShortcutAction::ProcessedRawCurrentLineView,
    ShortcutAction::SplitView,
    ShortcutAction::ToggleExplorer,
    ShortcutAction::ToggleTopMenu,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShortcutModifier {
    None,
    Platform,
    Ctrl,
    Alt,
    Super,
    Space,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShortcutBinding {
    pub(crate) key: KeyCode,
    pub(crate) shift: bool,
    pub(crate) modifier: ShortcutModifier,
}

#[derive(Clone, Debug)]
pub(crate) struct KeybindSettings {
    pub(crate) navigate_forward: ShortcutBinding,
    pub(crate) open_workspace: ShortcutBinding,
    pub(crate) save: ShortcutBinding,
    pub(crate) save_as: ShortcutBinding,
    pub(crate) undo: ShortcutBinding,
    pub(crate) redo: ShortcutBinding,
    pub(crate) zoom_in: ShortcutBinding,
    pub(crate) zoom_out: ShortcutBinding,
    pub(crate) plain_view: ShortcutBinding,
    pub(crate) processed_view: ShortcutBinding,
    pub(crate) processed_raw_current_line_view: ShortcutBinding,
    pub(crate) split_view: ShortcutBinding,
    pub(crate) toggle_explorer: ShortcutBinding,
    pub(crate) toggle_top_menu: ShortcutBinding,
}

impl Default for KeybindSettings {
    fn default() -> Self {
        Self {
            navigate_forward: ShortcutBinding::unmodified(KeyCode::Backquote),
            open_workspace: ShortcutBinding::platform(KeyCode::KeyO, false),
            save: ShortcutBinding::platform(KeyCode::KeyS, false),
            save_as: ShortcutBinding::platform(KeyCode::KeyS, true),
            undo: ShortcutBinding::platform(KeyCode::KeyZ, false),
            redo: ShortcutBinding::platform(KeyCode::KeyZ, true),
            zoom_in: ShortcutBinding::platform(KeyCode::Equal, false),
            zoom_out: ShortcutBinding::platform(KeyCode::Minus, false),
            plain_view: ShortcutBinding::platform(KeyCode::Digit3, false),
            processed_view: ShortcutBinding::platform(KeyCode::Digit2, false),
            processed_raw_current_line_view: ShortcutBinding::platform(KeyCode::Digit1, false),
            split_view: ShortcutBinding::platform(KeyCode::Digit4, false),
            toggle_explorer: ShortcutBinding::platform(KeyCode::KeyE, false),
            toggle_top_menu: ShortcutBinding::platform(KeyCode::KeyB, false),
        }
    }
}

impl ShortcutBinding {
    pub(crate) const fn unmodified(key: KeyCode) -> Self {
        Self {
            key,
            shift: false,
            modifier: ShortcutModifier::None,
        }
    }

    pub(crate) const fn platform(key: KeyCode, shift: bool) -> Self {
        Self {
            key,
            shift,
            modifier: ShortcutModifier::Platform,
        }
    }
}

impl KeybindSettings {
    pub(crate) fn binding(&self, action: ShortcutAction) -> ShortcutBinding {
        match action {
            ShortcutAction::NavigateForward => self.navigate_forward,
            ShortcutAction::OpenWorkspace => self.open_workspace,
            ShortcutAction::Save => self.save,
            ShortcutAction::SaveAs => self.save_as,
            ShortcutAction::Undo => self.undo,
            ShortcutAction::Redo => self.redo,
            ShortcutAction::ZoomIn => self.zoom_in,
            ShortcutAction::ZoomOut => self.zoom_out,
            ShortcutAction::PlainView => self.plain_view,
            ShortcutAction::ProcessedView => self.processed_view,
            ShortcutAction::ProcessedRawCurrentLineView => self.processed_raw_current_line_view,
            ShortcutAction::SplitView => self.split_view,
            ShortcutAction::ToggleExplorer => self.toggle_explorer,
            ShortcutAction::ToggleTopMenu => self.toggle_top_menu,
        }
    }

    pub(crate) fn set_binding(&mut self, action: ShortcutAction, binding: ShortcutBinding) {
        match action {
            ShortcutAction::NavigateForward => self.navigate_forward = binding,
            ShortcutAction::OpenWorkspace => self.open_workspace = binding,
            ShortcutAction::Save => self.save = binding,
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
            ShortcutAction::SplitView => self.split_view = binding,
            ShortcutAction::ToggleExplorer => self.toggle_explorer = binding,
            ShortcutAction::ToggleTopMenu => self.toggle_top_menu = binding,
        }
    }
}

pub(crate) fn shortcut_action_label(action: ShortcutAction) -> &'static str {
    match action {
        ShortcutAction::NavigateForward => "Navigate Forward",
        ShortcutAction::OpenWorkspace => "Open Workspace Folder",
        ShortcutAction::Save => "Save",
        ShortcutAction::SaveAs => "Save As Dialog",
        ShortcutAction::Undo => "Undo",
        ShortcutAction::Redo => "Redo",
        ShortcutAction::ZoomIn => "Zoom In",
        ShortcutAction::ZoomOut => "Zoom Out",
        ShortcutAction::PlainView => "Plain View Mode",
        ShortcutAction::ProcessedView => "Processed View Mode",
        ShortcutAction::ProcessedRawCurrentLineView => "Processed + Raw Current Line Mode",
        ShortcutAction::SplitView => "Dual Panel View Mode",
        ShortcutAction::ToggleExplorer => "Toggle Explorer",
        ShortcutAction::ToggleTopMenu => "Toggle Top Menu",
    }
}

pub(crate) fn shortcut_action_description(action: ShortcutAction) -> &'static str {
    match action {
        ShortcutAction::NavigateForward => "Move forward through page history",
        ShortcutAction::OpenWorkspace => "Open workspace folder",
        ShortcutAction::Save => "Save current file",
        ShortcutAction::SaveAs => "Save As dialog",
        ShortcutAction::Undo => "Undo",
        ShortcutAction::Redo => "Redo",
        ShortcutAction::ZoomIn => "Zoom in",
        ShortcutAction::ZoomOut => "Zoom out",
        ShortcutAction::PlainView => "Plain view mode",
        ShortcutAction::ProcessedView => "Processed view mode",
        ShortcutAction::ProcessedRawCurrentLineView => "Processed + raw current line mode",
        ShortcutAction::SplitView => "Dual panel view mode",
        ShortcutAction::ToggleExplorer => "Toggle explorer",
        ShortcutAction::ToggleTopMenu => "Toggle top menu",
    }
}

pub(crate) fn shortcut_action_settings_key(action: ShortcutAction) -> &'static str {
    match action {
        ShortcutAction::NavigateForward => "navigate_forward",
        ShortcutAction::OpenWorkspace => "open_workspace",
        ShortcutAction::Save => "save",
        ShortcutAction::SaveAs => "save_as",
        ShortcutAction::Undo => "undo",
        ShortcutAction::Redo => "redo",
        ShortcutAction::ZoomIn => "zoom_in",
        ShortcutAction::ZoomOut => "zoom_out",
        ShortcutAction::PlainView => "plain_view",
        ShortcutAction::ProcessedView => "processed_view",
        ShortcutAction::ProcessedRawCurrentLineView => "processed_raw_current_line_view",
        ShortcutAction::SplitView => "split_view",
        ShortcutAction::ToggleExplorer => "toggle_explorer",
        ShortcutAction::ToggleTopMenu => "toggle_top_menu",
    }
}

pub(crate) fn binding_key_name(key: KeyCode) -> Option<&'static str> {
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
        KeyCode::Backquote => Some("`"),
        KeyCode::Space => Some("Space"),
        _ => None,
    }
}

pub(crate) fn binding_key_from_name(name: &str) -> Option<KeyCode> {
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
        "`" | "BACKQUOTE" | "BACKTICK" => Some(KeyCode::Backquote),
        "SPACE" => Some(KeyCode::Space),
        _ => None,
    }
}

pub(crate) fn parse_binding_spec(spec: &str) -> Option<ShortcutBinding> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parts = trimmed
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let key_name = parts.last().copied()?;
    let mut shift = false;
    let mut modifier = ShortcutModifier::None;

    for modifier_name in parts.iter().take(parts.len().saturating_sub(1)) {
        match modifier_name.to_ascii_uppercase().as_str() {
            "SHIFT" => shift = true,
            "MOD" | "PLATFORM" | "CMD/CTRL" | "CTRL/CMD" | "COMMAND/CONTROL" => {
                modifier = ShortcutModifier::Platform
            }
            "CTRL" | "CONTROL" => modifier = ShortcutModifier::Ctrl,
            "ALT" | "OPTION" => modifier = ShortcutModifier::Alt,
            "SUPER" | "CMD" | "COMMAND" | "WIN" | "WINDOWS" => modifier = ShortcutModifier::Super,
            "SPACE" => modifier = ShortcutModifier::Space,
            _ => return None,
        }
    }

    let key = binding_key_from_name(key_name)?;
    Some(ShortcutBinding {
        key,
        shift,
        modifier,
    })
}

pub(crate) fn binding_spec(binding: ShortcutBinding) -> String {
    let mut parts = Vec::new();
    if binding.modifier != ShortcutModifier::None {
        parts.push(binding_modifier_spec(binding.modifier).to_string());
    }
    if binding.shift {
        parts.push("Shift".to_string());
    }
    parts.push(binding_key_name(binding.key).unwrap_or("?").to_string());
    parts.join("+")
}

pub(crate) fn binding_modifier_spec(modifier: ShortcutModifier) -> &'static str {
    match modifier {
        ShortcutModifier::None => "",
        ShortcutModifier::Platform => "Mod",
        ShortcutModifier::Ctrl => "Ctrl",
        ShortcutModifier::Alt => "Alt",
        ShortcutModifier::Super => "Super",
        ShortcutModifier::Space => "Space",
    }
}

pub(crate) fn binding_modifier_display(modifier: ShortcutModifier) -> &'static str {
    match modifier {
        ShortcutModifier::None => "",
        ShortcutModifier::Platform => "Cmd/Ctrl",
        ShortcutModifier::Ctrl => "Ctrl",
        ShortcutModifier::Alt => "Alt",
        ShortcutModifier::Super => "Super/Cmd",
        ShortcutModifier::Space => "Space",
    }
}

pub(crate) fn capture_shortcut_modifier(
    keys: &ButtonInput<KeyCode>,
    primary_key: KeyCode,
) -> Result<ShortcutModifier, &'static str> {
    let mut pressed = Vec::new();
    if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        pressed.push(ShortcutModifier::Ctrl);
    }
    if keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
        pressed.push(ShortcutModifier::Alt);
    }
    if keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]) {
        pressed.push(ShortcutModifier::Super);
    }
    if primary_key != KeyCode::Space && keys.pressed(KeyCode::Space) {
        pressed.push(ShortcutModifier::Space);
    }

    match pressed.as_slice() {
        [] => Ok(ShortcutModifier::None),
        [modifier] => Ok(*modifier),
        _ => Err("Use only one shortcut modifier: Ctrl, Alt, Super/Cmd, or Space."),
    }
}

pub(crate) fn binding_display(binding: ShortcutBinding) -> String {
    let mut parts = Vec::new();
    if binding.modifier != ShortcutModifier::None {
        parts.push(binding_modifier_display(binding.modifier).to_string());
    }
    if binding.shift {
        parts.push("Shift".to_string());
    }
    parts.push(binding_key_name(binding.key).unwrap_or("?").to_string());
    parts.join("+")
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KeybindRebindButton {
    pub(crate) action: ShortcutAction,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KeybindBindingLabel {
    pub(crate) action: ShortcutAction,
}

#[derive(Component)]
pub(crate) struct EditorScreenRoot;

#[derive(Component)]
pub(crate) struct WindowSurfaceRoot;

#[derive(Component)]
pub(crate) struct StatusLineRoot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SettingToggleLabel {
    pub(crate) action: SettingsAction,
}

#[derive(Component)]
pub(crate) struct SettingsScreenRoot;

#[derive(Component)]
pub(crate) struct KeybindsScreenRoot;

#[derive(Component)]
pub(crate) struct SettingsMenuScrollArea {
    pub(crate) screen: UiScreenState,
}

#[derive(Component)]
pub(crate) struct SettingsMenuScrollContent {
    pub(crate) screen: UiScreenState,
}

#[derive(Component)]
pub(crate) struct ThemeScreenRoot;

#[derive(Component)]
pub(crate) struct TopMenuSection;

#[derive(Component)]
pub(crate) struct ThemeOnlySettingControl;

pub(crate) fn window_surface_border_radius(show_system_titlebar: bool) -> BorderRadius {
    let radius = if show_system_titlebar {
        0.0
    } else {
        UNDECORATED_WINDOW_CORNER_RADIUS
    };
    BorderRadius::all(px(radius))
}

pub(crate) fn window_surface_overflow(show_system_titlebar: bool) -> Overflow {
    if show_system_titlebar {
        Overflow::visible()
    } else {
        Overflow::clip()
    }
}

pub(crate) fn window_surface_top_border_radius(
    round_left: bool,
    round_right: bool,
) -> BorderRadius {
    let radius = px(UNDECORATED_WINDOW_CORNER_RADIUS);
    BorderRadius::new(
        if round_left { radius } else { px(0.0) },
        if round_right { radius } else { px(0.0) },
        px(0.0),
        px(0.0),
    )
}

pub(crate) fn window_surface_bottom_border_radius(
    round_left: bool,
    round_right: bool,
) -> BorderRadius {
    let radius = px(UNDECORATED_WINDOW_CORNER_RADIUS);
    BorderRadius::new(
        px(0.0),
        px(0.0),
        if round_right { radius } else { px(0.0) },
        if round_left { radius } else { px(0.0) },
    )
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MarginEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SettingMarginLabel {
    pub(crate) edge: MarginEdge,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThemeColorChannel {
    Hue,
    Saturation,
    Red,
    Green,
    Blue,
    Value,
    Alpha,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ThemeColorRow {
    pub(crate) target: ThemeColorTarget,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ThemeColorLabel {
    pub(crate) channel: ThemeColorChannel,
}

#[derive(Component)]
pub(crate) struct ThemeScreenTitleLabel;

#[derive(Component)]
pub(crate) struct ThemeScreenDescriptionLabel;

#[derive(Component)]
pub(crate) struct ThemeColorNameLabel {
    pub(crate) target: ThemeColorTarget,
}

#[derive(Component)]
pub(crate) struct ThemeColorValueLabel {
    pub(crate) target: ThemeColorTarget,
}

#[derive(Component)]
pub(crate) struct ThemeColorPickerPanel;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ThemeColorPreviewSwatch {
    pub(crate) target: ThemeColorTarget,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ThemeColorPickerButton {
    pub(crate) target: ThemeColorTarget,
}

#[derive(Component)]
pub(crate) struct ThemeLinkHoverSettingRow;

#[derive(Component)]
pub(crate) struct ThemeLinkHoverValueLabel;

#[derive(Component)]
pub(crate) struct ThemeHueSatWheel;

#[derive(Component)]
pub(crate) struct ThemeHueSatCursor;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThemeSliderChannel {
    Hue,
    Saturation,
    Red,
    Green,
    Blue,
    Value,
    Alpha,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThemeColorTarget {
    AppBackground,
    TopMenuBackground,
    ExplorerBackground,
    ProcessedBackground,
    SelectionBackground,
    LinkFallback,
    LinkProp,
    LinkPlace,
    LinkCharacter,
    LinkFaction,
    LinkConcept,
}

impl ThemeColorTarget {
    pub(crate) fn screen_title(self) -> &'static str {
        if self.is_link_color() {
            "Link Colors"
        } else {
            "Theme"
        }
    }

    pub(crate) fn screen_description(self) -> &'static str {
        if self.is_link_color() {
            "Adjust processed-view link colors by YAML `type`. Unmapped types use Fallback, and hover uses the HSV value offset."
        } else {
            "Adjust editor shell colors, selection colors, and glass surfaces."
        }
    }

    pub(crate) fn color_label(self) -> &'static str {
        match self {
            Self::AppBackground => "App background",
            Self::TopMenuBackground => "Top menu",
            Self::ExplorerBackground => "Explorer",
            Self::ProcessedBackground => "Processed pane",
            Self::SelectionBackground => "Selection background",
            Self::LinkFallback => "Fallback",
            Self::LinkProp => "Prop",
            Self::LinkPlace => "Place",
            Self::LinkCharacter => "Character",
            Self::LinkFaction => "Faction",
            Self::LinkConcept => "Concept",
        }
    }

    pub(crate) fn status_label(self) -> &'static str {
        match self {
            Self::AppBackground => "app background",
            Self::TopMenuBackground => "top menu background",
            Self::ExplorerBackground => "explorer background",
            Self::ProcessedBackground => "processed pane background",
            Self::SelectionBackground => "selection background",
            Self::LinkFallback => "fallback link color",
            Self::LinkProp => "prop link color",
            Self::LinkPlace => "place link color",
            Self::LinkCharacter => "character link color",
            Self::LinkFaction => "faction link color",
            Self::LinkConcept => "concept link color",
        }
    }

    pub(crate) fn is_link_color(self) -> bool {
        matches!(
            self,
            Self::LinkFallback
                | Self::LinkProp
                | Self::LinkPlace
                | Self::LinkCharacter
                | Self::LinkFaction
                | Self::LinkConcept
        )
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ThemeColorSlider {
    pub(crate) channel: ThemeSliderChannel,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ThemeColorSliderKnob {
    pub(crate) channel: ThemeSliderChannel,
}

#[derive(Component)]
pub(crate) struct ThemeSelectionHsvLabel;

#[derive(Component)]
pub(crate) struct ThemeSelectionRgbLabel;

#[derive(Component)]
pub(crate) struct ThemeSelectionHexLabel;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HoveredProcessedLink {
    pub(crate) source_line: usize,
    pub(crate) raw_start_column: usize,
    pub(crate) raw_end_column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorkspaceSelectedRow {
    Folder(String),
    File(PathBuf),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorkspacePrompt {
    Create {
        input: String,
    },
    ChooseMarkdownTemplate {
        destination: PathBuf,
        templates: Vec<WorkspaceMarkdownTemplate>,
        selected: usize,
    },
    CreateLinkedMarkdown {
        source_path: PathBuf,
        range_start: Position,
        range_end: Position,
        label: String,
        filename: String,
        folders: Vec<PathBuf>,
        selected_folder: usize,
        expanded_folders: BTreeSet<PathBuf>,
        templates: Vec<WorkspaceMarkdownTemplate>,
        template_hint: Option<String>,
    },
    ChooseLinkedMarkdownTemplate {
        source_path: PathBuf,
        range_start: Position,
        range_end: Position,
        label: String,
        filename: String,
        folder: PathBuf,
        templates: Vec<WorkspaceMarkdownTemplate>,
        selected: usize,
    },
    Rename {
        target: WorkspaceSelectedRow,
        input: String,
    },
    Delete {
        target: WorkspaceSelectedRow,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkspaceMarkdownTemplate {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VimMode {
    Normal,
    Insert,
    VisualChar,
    VisualLine,
}

impl VimMode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::VisualChar => "VISUAL",
            Self::VisualLine => "V-LINE",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VimPendingOperator {
    Yank,
    Delete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum VimRegister {
    Characterwise(String),
    Linewise(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CommandMenu {
    pub(crate) input: String,
}

#[derive(Resource)]
pub(crate) struct EditorState {
    pub(crate) document: Document,
    pub(crate) parsed: Vec<ParsedLine>,
    pub(crate) document_format: DocumentFormat,
    pub(crate) cursor: Cursor,
    pub(crate) selection_anchor: Option<Position>,
    pub(crate) top_line: usize,
    pub(crate) processed_top_line: usize,
    pub(crate) processed_top_visual: usize,
    pub(crate) processed_preferred_column: Option<usize>,
    pub(crate) display_mode: DisplayMode,
    pub(crate) focused_panel: PanelKind,
    pub(crate) plain_horizontal_scroll: f32,
    pub(crate) processed_horizontal_scroll: f32,
    pub(crate) processed_header_scroll_progress: f32,
    pub(crate) processed_zoom_anchor_bias_px: f32,
    pub(crate) paths: DocumentPath,
    pub(crate) status_message: String,
    pub(crate) keybinds: KeybindSettings,
    pub(crate) pending_keybind_capture: Option<ShortcutAction>,
    pub(crate) pending_space_insert: bool,
    pub(crate) pending_space_combo_canceled: bool,
    pub(crate) vim_enabled: bool,
    pub(crate) vim_mode: VimMode,
    pub(crate) vim_suppress_next_insert_input: bool,
    pub(crate) vim_pending_operator: Option<VimPendingOperator>,
    pub(crate) vim_register: Option<VimRegister>,
    pub(crate) vim_visual_anchor: Option<Position>,
    pub(crate) vim_visual_head: Option<Position>,
    pub(crate) command_menu: Option<CommandMenu>,
    pub(crate) markdown_metadata_focus: Option<MarkdownMetadataField>,
    pub(crate) markdown_metadata_dropdown: Option<MarkdownMetadataField>,
    pub(crate) markdown_metadata_dropdown_highlight: usize,
    pub(crate) link_autocomplete: Option<LinkAutocomplete>,
    pub(crate) story_query_sheet: StoryQuerySheet,
    pub(crate) workspace_sidebar_visible: bool,
    pub(crate) top_menu_collapsed: bool,
    pub(crate) processed_link_color_mode: ProcessedLinkColorMode,
    pub(crate) processed_paginated: bool,
    pub(crate) formatting_marks_visible: bool,
    pub(crate) processed_glass: bool,
    pub(crate) explorer_glass: bool,
    pub(crate) settings_glass: bool,
    pub(crate) app_bg_rgba: Vec4,
    pub(crate) app_bg_color: Color,
    pub(crate) top_menu_bg_rgba: Vec4,
    pub(crate) top_menu_bg_color: Color,
    pub(crate) explorer_bg_rgba: Vec4,
    pub(crate) explorer_bg_color: Color,
    pub(crate) processed_bg_rgba: Vec4,
    pub(crate) processed_bg_color: Color,
    pub(crate) selection_bg_rgba: Vec4,
    pub(crate) selection_bg_color: Color,
    pub(crate) link_fallback_rgba: Vec4,
    pub(crate) link_fallback_color: Color,
    pub(crate) link_prop_rgba: Vec4,
    pub(crate) link_prop_color: Color,
    pub(crate) link_place_rgba: Vec4,
    pub(crate) link_place_color: Color,
    pub(crate) link_character_rgba: Vec4,
    pub(crate) link_character_color: Color,
    pub(crate) link_faction_rgba: Vec4,
    pub(crate) link_faction_color: Color,
    pub(crate) link_concept_rgba: Vec4,
    pub(crate) link_concept_color: Color,
    pub(crate) link_hover_hsv_value_adjustment: f32,
    pub(crate) theme_color_target: ThemeColorTarget,
    pub(crate) theme_color_picker_open: bool,
    pub(crate) show_system_titlebar: bool,
    pub(crate) caret_blink: Timer,
    pub(crate) caret_visible: bool,
    pub(crate) dialogue_double_space_newline: bool,
    pub(crate) non_dialogue_double_space_newline: bool,
    pub(crate) page_margin_left: f32,
    pub(crate) page_margin_right: f32,
    pub(crate) page_margin_top: f32,
    pub(crate) page_margin_bottom: f32,
    pub(crate) zoom: f32,
    pub(crate) measured_line_step: f32,
    pub(crate) processed_cache: Option<ProcessedCache>,
    pub(crate) processed_cache_dirty_from_line: Option<usize>,
    pub(crate) canvas_document: Option<CanvasDocument>,
    pub(crate) canvas_parse_error: Option<String>,
    pub(crate) canvas_version: u64,
    pub(crate) canvas_pan: Vec2,
    pub(crate) canvas_view_needs_centering: bool,
    pub(crate) canvas_editing_node_id: Option<String>,
    pub(crate) canvas_text_cursor: Cursor,
    pub(crate) canvas_text_selection_anchor: Option<Position>,
    pub(crate) canvas_text_edit_undo_snapshot: Option<EditorHistorySnapshot>,
    pub(crate) canvas_text_suppress_next_insert_input: bool,
    pub(crate) workspace_root: Option<PathBuf>,
    pub(crate) story_index: Option<EditorStoryIndex>,
    pub(crate) workspace_folders: Vec<WorkspaceFolderEntry>,
    pub(crate) workspace_files: Vec<WorkspaceFileEntry>,
    pub(crate) workspace_active_file: Option<usize>,
    pub(crate) workspace_selected_row: Option<WorkspaceSelectedRow>,
    pub(crate) workspace_focused: bool,
    pub(crate) workspace_prompt: Option<WorkspacePrompt>,
    pub(crate) workspace_expanded_folders: BTreeSet<String>,
    pub(crate) script_link_target_types: BTreeMap<String, String>,
    pub(crate) missing_script_link_targets: BTreeSet<String>,
    pub(crate) hovered_processed_link: Option<HoveredProcessedLink>,
    pub(crate) workspace_ui_dirty: bool,
    pub(crate) undo_history: Vec<EditorHistorySnapshot>,
    pub(crate) redo_history: Vec<EditorHistorySnapshot>,
    pub(crate) document_navigation_history: Vec<DocumentNavigationEntry>,
    pub(crate) document_navigation_forward_history: Vec<DocumentNavigationEntry>,
}

#[derive(Resource, Default)]
pub(crate) struct EditorImageCache {
    pub(crate) entries: BTreeMap<PathBuf, CachedProcessedImage>,
}

#[derive(Clone)]
pub(crate) struct CachedProcessedImage {
    pub(crate) modified: Option<SystemTime>,
    pub(crate) result: CachedProcessedImageResult,
}

#[derive(Clone)]
pub(crate) enum CachedProcessedImageResult {
    Loaded { handle: Handle<Image>, size: UVec2 },
    Failed,
}

#[derive(Clone)]
pub(crate) enum ProcessedImageLookup {
    Loaded { handle: Handle<Image>, size: UVec2 },
    Failed,
}

#[derive(Clone)]
pub(crate) struct EditorHistorySnapshot {
    pub(crate) document: Document,
    pub(crate) cursor: Cursor,
    pub(crate) top_line: usize,
    pub(crate) processed_top_line: usize,
    pub(crate) processed_top_visual: usize,
    pub(crate) plain_horizontal_scroll: f32,
    pub(crate) processed_horizontal_scroll: f32,
    pub(crate) processed_header_scroll_progress: f32,
    pub(crate) processed_zoom_anchor_bias_px: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct DocumentNavigationEntry {
    pub(crate) path: PathBuf,
    pub(crate) cursor: Cursor,
    pub(crate) selection_anchor: Option<Position>,
    pub(crate) top_line: usize,
    pub(crate) processed_top_line: usize,
    pub(crate) processed_top_visual: usize,
    pub(crate) plain_horizontal_scroll: f32,
    pub(crate) processed_horizontal_scroll: f32,
    pub(crate) processed_header_scroll_progress: f32,
    pub(crate) processed_zoom_anchor_bias_px: f32,
    pub(crate) display_mode: DisplayMode,
    pub(crate) focused_panel: PanelKind,
    pub(crate) zoom: f32,
    pub(crate) canvas_pan: Vec2,
}

#[derive(Resource, Default)]
pub(crate) struct DialogState {
    pub(crate) pending: Option<PendingDialog>,
    pub(crate) opened_at: Option<Instant>,
    pub(crate) last_watchdog_log_at: Option<Instant>,
    pub(crate) poll_count: u64,
}

#[derive(Resource, Default)]
pub(crate) struct NativeGlassState {
    pub(crate) active: bool,
    pub(crate) initialized: bool,
}

#[derive(Resource, Default)]
pub(crate) struct MiddleAutoscrollState {
    pub(crate) panel: Option<PanelKind>,
    pub(crate) anchor_cursor_position: Vec2,
    pub(crate) plain_vertical_remainder_lines: f32,
    pub(crate) suppress_next_left_click: bool,
}

#[derive(Resource, Default, Clone, Copy, Debug)]
pub(crate) struct NavigationRepeatState {
    pub(crate) active_arrow: Option<KeyCode>,
    pub(crate) repeat_cooldown_secs: f32,
}

#[derive(Resource, Default, Clone, Copy, Debug)]
pub(crate) struct WorkspaceSelectionRepeatState {
    pub(crate) active_arrow: Option<KeyCode>,
    pub(crate) repeat_cooldown_secs: f32,
}

#[derive(Resource, Clone, Copy, Debug)]
pub(crate) struct PanelLayoutState {
    pub(crate) workspace_width_px: f32,
    pub(crate) plain_ratio: f32,
}

#[derive(Resource, Default, Clone, Copy, Debug)]
pub(crate) struct ResolvedPanelWidths {
    plain: Option<f32>,
    processed: Option<f32>,
}

impl ResolvedPanelWidths {
    pub(crate) fn set(&mut self, plain: f32, processed: f32) {
        self.plain = Some(plain.max(0.0));
        self.processed = Some(processed.max(0.0));
    }

    pub(crate) fn panel_size(&self, kind: PanelKind, computed: &ComputedNode) -> Vec2 {
        let inverse_scale = computed.inverse_scale_factor();
        let mut logical_size = computed.size() * inverse_scale;
        let resolved_width = match kind {
            PanelKind::Plain => self.plain,
            PanelKind::Processed => self.processed,
        };
        if let Some(width) = resolved_width {
            logical_size.x = width;
        }
        logical_size
    }
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
pub(crate) struct PanelSplitterDragState {
    pub(crate) active: Option<PanelSplitter>,
    pub(crate) last_cursor_x: Option<f32>,
    pub(crate) suppress_next_left_click: bool,
}

pub(crate) type DialogPathResult = Result<Option<PathBuf>, String>;

pub(crate) enum PendingDialog {
    Workspace(Arc<Mutex<mpsc::Receiver<DialogPathResult>>>),
    Save(Arc<Mutex<mpsc::Receiver<DialogPathResult>>>),
    ExportPdf(Arc<Mutex<mpsc::Receiver<DialogPathResult>>>),
}

pub(crate) struct DialogMainThreadMarker;

#[derive(Clone, Debug)]
pub(crate) struct PersistentSettings {
    pub(crate) dialogue_double_space_newline: bool,
    pub(crate) non_dialogue_double_space_newline: bool,
    pub(crate) show_system_titlebar: bool,
    pub(crate) page_margin_left: f32,
    pub(crate) page_margin_right: f32,
    pub(crate) page_margin_top: f32,
    pub(crate) page_margin_bottom: f32,
    pub(crate) workspace_root_path: Option<String>,
    pub(crate) vim_mode_enabled: bool,
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
            vim_mode_enabled: false,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PersistentUiState {
    pub(crate) workspace_sidebar_visible: bool,
    pub(crate) top_menu_collapsed: bool,
    pub(crate) processed_link_color_mode: ProcessedLinkColorMode,
    pub(crate) processed_paginated: bool,
    pub(crate) formatting_marks_visible: bool,
}

impl Default for PersistentUiState {
    fn default() -> Self {
        Self {
            workspace_sidebar_visible: true,
            top_menu_collapsed: false,
            processed_link_color_mode: ProcessedLinkColorMode::Colored,
            processed_paginated: true,
            formatting_marks_visible: false,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ThemeSettings {
    pub(crate) app_background: Vec4,
    pub(crate) top_menu_background: Vec4,
    pub(crate) explorer_background: Vec4,
    pub(crate) processed_background: Vec4,
    pub(crate) selection_background: Vec4,
    pub(crate) link_fallback: Vec4,
    pub(crate) link_prop: Vec4,
    pub(crate) link_place: Vec4,
    pub(crate) link_character: Vec4,
    pub(crate) link_faction: Vec4,
    pub(crate) link_concept: Vec4,
    pub(crate) link_hover_hsv_value_adjustment: f32,
    pub(crate) processed_glass: bool,
    pub(crate) explorer_glass: bool,
    pub(crate) settings_glass: bool,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            app_background: Vec4::new(0.79, 0.80, 0.82, 1.0),
            top_menu_background: Vec4::new(0.79, 0.80, 0.82, 1.0),
            explorer_background: Vec4::new(0.86, 0.87, 0.89, 1.0),
            processed_background: Vec4::new(0.82, 0.83, 0.84, 1.0),
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
            settings_glass: false,
        }
    }
}

impl ThemeSettings {
    pub(crate) fn app_background_clamped(&self) -> Vec4 {
        Vec4::new(
            self.app_background.x.clamp(0.0, 1.0),
            self.app_background.y.clamp(0.0, 1.0),
            self.app_background.z.clamp(0.0, 1.0),
            self.app_background.w.clamp(0.0, 1.0),
        )
    }

    pub(crate) fn app_background_color(&self) -> Color {
        let rgba = self.app_background_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    pub(crate) fn top_menu_background_clamped(&self) -> Vec4 {
        Vec4::new(
            self.top_menu_background.x.clamp(0.0, 1.0),
            self.top_menu_background.y.clamp(0.0, 1.0),
            self.top_menu_background.z.clamp(0.0, 1.0),
            self.top_menu_background.w.clamp(0.0, 1.0),
        )
    }

    pub(crate) fn top_menu_background_color(&self) -> Color {
        let rgba = self.top_menu_background_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    pub(crate) fn explorer_background_clamped(&self) -> Vec4 {
        Vec4::new(
            self.explorer_background.x.clamp(0.0, 1.0),
            self.explorer_background.y.clamp(0.0, 1.0),
            self.explorer_background.z.clamp(0.0, 1.0),
            self.explorer_background.w.clamp(0.0, 1.0),
        )
    }

    pub(crate) fn processed_background_clamped(&self) -> Vec4 {
        Vec4::new(
            self.processed_background.x.clamp(0.0, 1.0),
            self.processed_background.y.clamp(0.0, 1.0),
            self.processed_background.z.clamp(0.0, 1.0),
            self.processed_background.w.clamp(0.0, 1.0),
        )
    }

    pub(crate) fn explorer_background_color(&self) -> Color {
        let rgba = self.explorer_background_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    pub(crate) fn processed_background_color(&self) -> Color {
        let rgba = self.processed_background_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    pub(crate) fn selection_background_clamped(&self) -> Vec4 {
        Vec4::new(
            self.selection_background.x.clamp(0.0, 1.0),
            self.selection_background.y.clamp(0.0, 1.0),
            self.selection_background.z.clamp(0.0, 1.0),
            self.selection_background.w.clamp(0.0, 1.0),
        )
    }

    pub(crate) fn selection_background_color(&self) -> Color {
        let rgba = self.selection_background_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    pub(crate) fn link_fallback_clamped(&self) -> Vec4 {
        Vec4::new(
            self.link_fallback.x.clamp(0.0, 1.0),
            self.link_fallback.y.clamp(0.0, 1.0),
            self.link_fallback.z.clamp(0.0, 1.0),
            self.link_fallback.w.clamp(0.0, 1.0),
        )
    }

    pub(crate) fn link_fallback_color(&self) -> Color {
        let rgba = self.link_fallback_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    pub(crate) fn link_prop_clamped(&self) -> Vec4 {
        Vec4::new(
            self.link_prop.x.clamp(0.0, 1.0),
            self.link_prop.y.clamp(0.0, 1.0),
            self.link_prop.z.clamp(0.0, 1.0),
            self.link_prop.w.clamp(0.0, 1.0),
        )
    }

    pub(crate) fn link_prop_color(&self) -> Color {
        let rgba = self.link_prop_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    pub(crate) fn link_place_clamped(&self) -> Vec4 {
        Vec4::new(
            self.link_place.x.clamp(0.0, 1.0),
            self.link_place.y.clamp(0.0, 1.0),
            self.link_place.z.clamp(0.0, 1.0),
            self.link_place.w.clamp(0.0, 1.0),
        )
    }

    pub(crate) fn link_place_color(&self) -> Color {
        let rgba = self.link_place_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    pub(crate) fn link_character_clamped(&self) -> Vec4 {
        Vec4::new(
            self.link_character.x.clamp(0.0, 1.0),
            self.link_character.y.clamp(0.0, 1.0),
            self.link_character.z.clamp(0.0, 1.0),
            self.link_character.w.clamp(0.0, 1.0),
        )
    }

    pub(crate) fn link_character_color(&self) -> Color {
        let rgba = self.link_character_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    pub(crate) fn link_faction_clamped(&self) -> Vec4 {
        Vec4::new(
            self.link_faction.x.clamp(0.0, 1.0),
            self.link_faction.y.clamp(0.0, 1.0),
            self.link_faction.z.clamp(0.0, 1.0),
            self.link_faction.w.clamp(0.0, 1.0),
        )
    }

    pub(crate) fn link_faction_color(&self) -> Color {
        let rgba = self.link_faction_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    pub(crate) fn link_concept_clamped(&self) -> Vec4 {
        Vec4::new(
            self.link_concept.x.clamp(0.0, 1.0),
            self.link_concept.y.clamp(0.0, 1.0),
            self.link_concept.z.clamp(0.0, 1.0),
            self.link_concept.w.clamp(0.0, 1.0),
        )
    }

    pub(crate) fn link_concept_color(&self) -> Color {
        let rgba = self.link_concept_clamped();
        Color::srgba(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    pub(crate) fn link_hover_hsv_value_adjustment_clamped(&self) -> f32 {
        self.link_hover_hsv_value_adjustment
            .clamp(0.0, LINK_HOVER_HSV_VALUE_MAX)
    }
}

#[derive(Resource, Clone)]
pub(crate) struct EditorFonts {
    pub(crate) regular: Handle<Font>,
    pub(crate) bold: Handle<Font>,
    pub(crate) italic: Handle<Font>,
    pub(crate) bold_italic: Handle<Font>,
    pub(crate) markdown_regular: Handle<Font>,
    pub(crate) markdown_bold: Handle<Font>,
    pub(crate) markdown_italic: Handle<Font>,
    pub(crate) markdown_bold_italic: Handle<Font>,
}

#[derive(Resource, Clone)]
pub(crate) struct ChecklistIcons {
    pub(crate) unchecked: Handle<Image>,
    pub(crate) checked: Handle<Image>,
}

#[derive(Resource, Clone)]
pub(crate) struct ThemePickerAssets {
    pub(crate) hue_sat_wheel: Handle<Image>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FontVariant {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

pub(crate) fn text_font_attributes_for_variant(variant: FontVariant) -> (FontWeight, FontStyle) {
    match variant {
        FontVariant::Regular => (FontWeight::NORMAL, FontStyle::Normal),
        FontVariant::Bold => (FontWeight::BOLD, FontStyle::Normal),
        FontVariant::Italic => (FontWeight::NORMAL, FontStyle::Italic),
        FontVariant::BoldItalic => (FontWeight::BOLD, FontStyle::Italic),
    }
}

pub(crate) fn apply_font_variant_to_text_font(
    text_font: &mut TextFont,
    fonts: &EditorFonts,
    variant: FontVariant,
    format: DocumentFormat,
) {
    let next_font = font_for_variant_with_format(fonts, variant, format).into();
    if text_font.font != next_font {
        text_font.font = next_font;
    }

    let (next_weight, next_style) = text_font_attributes_for_variant(variant);
    if text_font.weight != next_weight {
        text_font.weight = next_weight;
    }
    if text_font.style != next_style {
        text_font.style = next_style;
    }
}

pub(crate) fn text_font_for_variant(
    fonts: &EditorFonts,
    variant: FontVariant,
    format: DocumentFormat,
    font_size: f32,
) -> TextFont {
    let mut text_font = TextFont {
        font_size: FontSize::Px(font_size),
        ..default()
    };
    apply_font_variant_to_text_font(&mut text_font, fonts, variant, format);
    text_font
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct InlineTextStyle {
    pub(crate) bold: bool,
    pub(crate) italic: bool,
}

pub(crate) fn apply_inline_style_to_font_variant(
    base: FontVariant,
    inline_style: InlineTextStyle,
) -> FontVariant {
    let base_bold = matches!(base, FontVariant::Bold | FontVariant::BoldItalic);
    let base_italic = matches!(base, FontVariant::Italic | FontVariant::BoldItalic);
    match (
        base_bold || inline_style.bold,
        base_italic || inline_style.italic,
    ) {
        (true, true) => FontVariant::BoldItalic,
        (true, false) => FontVariant::Bold,
        (false, true) => FontVariant::Italic,
        (false, false) => FontVariant::Regular,
    }
}

impl DialogState {
    pub(crate) fn begin_pending(&mut self, pending: PendingDialog) {
        let now = Instant::now();
        self.pending = Some(pending);
        self.opened_at = Some(now);
        self.last_watchdog_log_at = Some(now);
        self.poll_count = 0;
    }

    pub(crate) fn clear_pending(&mut self) {
        self.pending = None;
        self.opened_at = None;
        self.last_watchdog_log_at = None;
        self.poll_count = 0;
    }
}

impl MiddleAutoscrollState {
    pub(crate) fn is_active(&self) -> bool {
        self.panel.is_some()
    }

    pub(crate) fn start(&mut self, panel: PanelKind, anchor_cursor_position: Vec2) {
        self.panel = Some(panel);
        self.anchor_cursor_position = anchor_cursor_position;
        self.plain_vertical_remainder_lines = 0.0;
        self.suppress_next_left_click = false;
    }

    pub(crate) fn stop(&mut self) {
        self.panel = None;
        self.plain_vertical_remainder_lines = 0.0;
    }
}

impl PendingDialog {
    pub(crate) fn kind_name(&self) -> &'static str {
        match self {
            PendingDialog::Workspace(_) => "workspace",
            PendingDialog::Save(_) => "save",
            PendingDialog::ExportPdf(_) => "PDF export",
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
            processed_preferred_column: None,
            display_mode: DisplayMode::Split,
            focused_panel: PanelKind::Plain,
            plain_horizontal_scroll: 0.0,
            processed_horizontal_scroll: 0.0,
            processed_header_scroll_progress: 0.0,
            processed_zoom_anchor_bias_px: 0.0,
            paths,
            status_message,
            keybinds,
            pending_keybind_capture: None,
            pending_space_insert: false,
            pending_space_combo_canceled: false,
            vim_enabled: settings.vim_mode_enabled,
            vim_mode: VimMode::Normal,
            vim_suppress_next_insert_input: false,
            vim_pending_operator: None,
            vim_register: None,
            vim_visual_anchor: None,
            vim_visual_head: None,
            command_menu: None,
            markdown_metadata_focus: None,
            markdown_metadata_dropdown: None,
            markdown_metadata_dropdown_highlight: 0,
            link_autocomplete: None,
            story_query_sheet: StoryQuerySheet::default(),
            workspace_sidebar_visible: ui_state.workspace_sidebar_visible,
            top_menu_collapsed: ui_state.top_menu_collapsed,
            processed_link_color_mode: ui_state.processed_link_color_mode,
            processed_paginated: ui_state.processed_paginated,
            formatting_marks_visible: ui_state.formatting_marks_visible,
            processed_glass: theme_settings.processed_glass,
            explorer_glass: theme_settings.explorer_glass,
            settings_glass: theme_settings.settings_glass,
            app_bg_rgba: theme_settings.app_background_clamped(),
            app_bg_color: theme_settings.app_background_color(),
            top_menu_bg_rgba: theme_settings.top_menu_background_clamped(),
            top_menu_bg_color: theme_settings.top_menu_background_color(),
            explorer_bg_rgba: theme_settings.explorer_background_clamped(),
            explorer_bg_color: theme_settings.explorer_background_color(),
            processed_bg_rgba: theme_settings.processed_background_clamped(),
            processed_bg_color: theme_settings.processed_background_color(),
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
            theme_color_target: ThemeColorTarget::AppBackground,
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
            canvas_document: None,
            canvas_parse_error: None,
            canvas_version: 0,
            canvas_pan: Vec2::ZERO,
            canvas_view_needs_centering: false,
            canvas_editing_node_id: None,
            canvas_text_cursor: Cursor::default(),
            canvas_text_selection_anchor: None,
            canvas_text_edit_undo_snapshot: None,
            canvas_text_suppress_next_insert_input: false,
            workspace_root: None,
            story_index: None,
            workspace_folders: Vec::new(),
            workspace_files: Vec::new(),
            workspace_active_file: None,
            workspace_selected_row: None,
            workspace_focused: false,
            workspace_prompt: None,
            workspace_expanded_folders: BTreeSet::new(),
            script_link_target_types: BTreeMap::new(),
            missing_script_link_targets: BTreeSet::new(),
            hovered_processed_link: None,
            workspace_ui_dirty: true,
            undo_history: Vec::new(),
            redo_history: Vec::new(),
            document_navigation_history: Vec::with_capacity(DOCUMENT_NAVIGATION_HISTORY_LIMIT),
            document_navigation_forward_history: Vec::with_capacity(
                DOCUMENT_NAVIGATION_HISTORY_LIMIT,
            ),
        };
        normalize_page_margins(&mut next);
        let initial_status = next.status_message.clone();
        apply_initial_workspace_root(&mut next, &initial_status, saved_workspace_root.as_deref());
        if next.document_format == DocumentFormat::Canvas {
            next.display_mode = DisplayMode::Processed;
            next.focused_panel = PanelKind::Processed;
            next.sync_canvas_document();
            next.reset_canvas_view_to_content();
        }
        next
    }
}

impl EditorState {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub(crate) fn any_glass_enabled(&self) -> bool {
        self.processed_glass || self.explorer_glass || self.settings_glass
    }

    pub(crate) fn set_display_mode(&mut self, mode: DisplayMode) -> bool {
        if self.display_mode == mode {
            return false;
        }

        self.display_mode = mode;
        if self.document_format == DocumentFormat::Canvas {
            self.focused_panel = PanelKind::Processed;
            self.canvas_version = self.canvas_version.saturating_add(1);
            self.reset_blink();
            return true;
        }

        if !self.panel_visible(self.focused_panel) {
            self.focused_panel = self.active_panel_for_display_mode();
        }
        self.processed_preferred_column = None;
        self.reset_blink();
        true
    }

    pub(crate) fn panel_layout_display_mode(&self) -> DisplayMode {
        if self.document_format == DocumentFormat::Canvas {
            DisplayMode::Processed
        } else {
            self.display_mode
        }
    }

    pub(crate) fn active_panel_for_display_mode(&self) -> PanelKind {
        if self.document_format == DocumentFormat::Canvas {
            return PanelKind::Processed;
        }

        match self.display_mode {
            DisplayMode::Split => self.focused_panel,
            DisplayMode::Plain => PanelKind::Plain,
            DisplayMode::Processed | DisplayMode::ProcessedRawCurrentLine => PanelKind::Processed,
        }
    }

    pub(crate) fn panel_visible(&self, panel: PanelKind) -> bool {
        if self.document_format == DocumentFormat::Canvas {
            return panel == PanelKind::Processed;
        }

        self.display_mode.panel_visible(panel)
    }

    pub(crate) fn set_zoom(&mut self, zoom: f32) {
        let (min_zoom, max_zoom) = if self.document_format == DocumentFormat::Canvas {
            (CANVAS_ZOOM_MIN, CANVAS_ZOOM_MAX)
        } else {
            (ZOOM_MIN, ZOOM_MAX)
        };
        self.zoom = zoom.clamp(min_zoom, max_zoom);
        self.measured_line_step = scaled_line_height(self);
        self.reset_blink();
    }

    pub(crate) fn zoom_percent(&self) -> u32 {
        (self.zoom * 100.0).round() as u32
    }

    pub(crate) fn reparse(&mut self) {
        self.parsed = parse_document_with_format(&self.document, self.document_format);
        self.sync_canvas_document();
        self.missing_script_link_targets.clear();
        self.mark_processed_cache_dirty_from(0);
    }

    pub(crate) fn reparse_with_dirty_hint(&mut self, dirty_line: usize) {
        self.parsed = parse_document_with_format(&self.document, self.document_format);
        self.sync_canvas_document();
        self.missing_script_link_targets.clear();
        self.mark_processed_cache_dirty_from(dirty_line);
    }

    pub(crate) fn mark_processed_cache_dirty_from(&mut self, source_line: usize) {
        let dirty_line = source_line.min(self.document.line_count().saturating_sub(1));
        self.processed_cache_dirty_from_line = Some(
            self.processed_cache_dirty_from_line
                .map_or(dirty_line, |current| current.min(dirty_line)),
        );
    }

    pub(crate) fn reset_blink(&mut self) {
        self.caret_blink.reset();
        self.caret_visible = true;
    }

    pub(crate) fn selection_bounds(&self) -> Option<(Position, Position)> {
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

    pub(crate) fn delete_selection(&mut self) -> Option<Position> {
        let (start, end) = self.selection_bounds()?;
        let next = self.document.delete_range(start, end);
        self.set_cursor(next, true);
        Some(next)
    }

    pub(crate) fn max_top_line(&self, _visible_lines: usize) -> usize {
        self.document.line_count().saturating_sub(1)
    }

    pub(crate) fn clamp_scroll(&mut self, visible_lines: usize) {
        let max_top = self.max_top_line(visible_lines);
        self.top_line = self.top_line.min(max_top);
    }

    pub(crate) fn clamp_processed_top_line(&mut self) {
        let max_top = self.document.line_count().saturating_sub(1);
        self.processed_top_line = self.processed_top_line.min(max_top);
    }

    pub(crate) fn clamp_horizontal_scrolls(
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

    pub(crate) fn scroll_by(&mut self, line_delta: isize, visible_lines: usize) {
        let max_top = self.max_top_line(visible_lines) as isize;
        let next = (self.top_line as isize + line_delta).clamp(0, max_top);
        self.top_line = next as usize;
        self.processed_top_line = self.top_line;
    }

    pub(crate) fn ensure_cursor_visible(&mut self, visible_lines: usize) {
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

    pub(crate) fn set_cursor(&mut self, position: Position, update_preferred: bool) {
        self.set_cursor_with_selection(position, update_preferred, false);
    }

    pub(crate) fn set_cursor_with_selection(
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
        self.processed_preferred_column = None;

        if update_preferred {
            self.cursor.set_position(clamped);
        } else {
            self.cursor.position = clamped;
        }

        self.selection_anchor = anchor;
        if self
            .selection_anchor
            .is_some_and(|start| start == self.cursor.position)
        {
            self.selection_anchor = None;
        }

        self.reset_blink();
    }

    pub(crate) fn save_to_path(&mut self, path: PathBuf) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if detect_document_format(&path, &self.document) == DocumentFormat::Markdown
            && let Some(document) = normalize_markdown_front_matter_target(&self.document)
        {
            self.document = document;
            self.cursor.position = self.document.clamp_position(self.cursor.position);
            self.cursor.preferred_column = self
                .cursor
                .preferred_column
                .min(self.document.line_len_chars(self.cursor.position.line));
        }

        match self.document.save(&path) {
            Ok(()) => {
                self.paths.load_path = path.clone();
                self.paths.save_path = path.clone();
                self.document_format = detect_document_format(&path, &self.document);
                self.set_zoom(self.zoom);
                self.reparse();
                self.refresh_workspace_after_path_change();
                self.status_message = format!("Saved {}", status_path_label(&path));
            }
            Err(error) => {
                self.status_message =
                    format!("Save failed for {}: {error}", status_path_label(&path));
            }
        }
    }

    pub(crate) fn save_current(&mut self) {
        self.save_to_path(self.paths.save_path.clone());
    }

    pub(crate) fn load_from_path(&mut self, path: PathBuf) -> bool {
        match Document::load(&path) {
            Ok(document) => {
                let document_format = detect_document_format(&path, &document);
                self.document = document;
                self.document_format = document_format;
                self.set_zoom(self.zoom);
                self.clear_script_link_target_cache();
                self.reparse();
                self.cursor = Cursor::default();
                self.selection_anchor = None;
                self.vim_pending_operator = None;
                self.vim_visual_anchor = None;
                self.vim_visual_head = None;
                self.vim_suppress_next_insert_input = false;
                self.command_menu = None;
                self.clear_markdown_metadata_focus();
                self.link_autocomplete = None;
                self.canvas_editing_node_id = None;
                self.canvas_text_cursor = Cursor::default();
                self.canvas_text_selection_anchor = None;
                self.canvas_text_edit_undo_snapshot = None;
                self.canvas_text_suppress_next_insert_input = false;
                self.top_line = 0;
                self.processed_top_line = 0;
                self.processed_top_visual = 0;
                self.processed_preferred_column = None;
                self.plain_horizontal_scroll = 0.0;
                self.processed_horizontal_scroll = 0.0;
                self.processed_header_scroll_progress = 0.0;
                self.processed_zoom_anchor_bias_px = 0.0;
                self.canvas_pan = Vec2::ZERO;
                if document_format == DocumentFormat::Canvas {
                    self.display_mode = DisplayMode::Processed;
                    self.focused_panel = PanelKind::Processed;
                    self.reset_canvas_view_to_content();
                }
                self.clear_history();
                self.paths.load_path = path.clone();
                self.paths.save_path = path.clone();
                self.status_message = format!(
                    "Loaded {} ({}).",
                    status_path_label(&path),
                    document_format_label(self.document_format)
                );
                self.sync_workspace_active_file();
                self.reset_blink();
                true
            }
            Err(error) => {
                self.status_message =
                    format!("Load failed for {}: {error}", status_path_label(&path));
                false
            }
        }
    }

    pub(crate) fn history_snapshot(&self) -> EditorHistorySnapshot {
        EditorHistorySnapshot {
            document: self.document.clone(),
            cursor: self.cursor,
            top_line: self.top_line,
            processed_top_line: self.processed_top_line,
            processed_top_visual: self.processed_top_visual,
            plain_horizontal_scroll: self.plain_horizontal_scroll,
            processed_horizontal_scroll: self.processed_horizontal_scroll,
            processed_header_scroll_progress: self.processed_header_scroll_progress,
            processed_zoom_anchor_bias_px: self.processed_zoom_anchor_bias_px,
        }
    }

    pub(crate) fn push_history_snapshot(
        history: &mut Vec<EditorHistorySnapshot>,
        snapshot: EditorHistorySnapshot,
    ) {
        if history.len() >= HISTORY_LIMIT {
            history.remove(0);
        }
        history.push(snapshot);
    }

    pub(crate) fn push_undo_snapshot(&mut self, snapshot: EditorHistorySnapshot) {
        Self::push_history_snapshot(&mut self.undo_history, snapshot);
        self.redo_history.clear();
    }

    pub(crate) fn apply_history_snapshot(
        &mut self,
        snapshot: EditorHistorySnapshot,
        visible_lines: usize,
        plain_panel_size: Option<Vec2>,
        processed_panel_size: Option<Vec2>,
    ) {
        self.document = snapshot.document;
        self.parsed = parse_document_with_format(&self.document, self.document_format);
        self.sync_canvas_document();
        self.processed_cache = None;
        self.processed_cache_dirty_from_line = Some(0);

        self.cursor = snapshot.cursor;
        self.cursor.position = self.document.clamp_position(self.cursor.position);
        self.cursor.preferred_column = self
            .cursor
            .preferred_column
            .min(self.document.line_len_chars(self.cursor.position.line));
        self.selection_anchor = None;
        self.processed_preferred_column = None;

        self.top_line = snapshot.top_line;
        self.processed_top_line = snapshot.processed_top_line;
        self.processed_top_visual = snapshot.processed_top_visual;
        self.plain_horizontal_scroll = snapshot.plain_horizontal_scroll;
        self.processed_horizontal_scroll = snapshot.processed_horizontal_scroll;
        self.processed_header_scroll_progress = snapshot.processed_header_scroll_progress;
        self.processed_zoom_anchor_bias_px = snapshot.processed_zoom_anchor_bias_px;
        if self.document_format == DocumentFormat::Canvas {
            self.display_mode = DisplayMode::Processed;
            self.focused_panel = PanelKind::Processed;
            self.reset_canvas_view_to_content();
        }
        self.clamp_scroll(visible_lines);
        self.clamp_processed_top_line();
        self.clamp_horizontal_scrolls(plain_panel_size, processed_panel_size);
        self.reset_blink();
    }

    pub(crate) fn undo(
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

    pub(crate) fn redo(
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

    pub(crate) fn clear_history(&mut self) {
        self.undo_history.clear();
        self.redo_history.clear();
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ProcessedPageGeometry {
    pub(crate) paper_left: f32,
    pub(crate) paper_top: f32,
    pub(crate) paper_width: f32,
    pub(crate) paper_height: f32,
    pub(crate) page_gap: f32,
    pub(crate) text_left: f32,
    pub(crate) text_top: f32,
    pub(crate) text_width: f32,
    pub(crate) text_height: f32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ProcessedPageLayout {
    pub(crate) geometry: ProcessedPageGeometry,
    pub(crate) wrap_columns: usize,
    pub(crate) lines_per_page: usize,
    pub(crate) spacer_lines: usize,
    pub(crate) page_step_lines: usize,
}

pub(crate) fn document_format_label(format: DocumentFormat) -> &'static str {
    match format {
        DocumentFormat::Fountain => "Fountain",
        DocumentFormat::Markdown => "Markdown",
        DocumentFormat::Canvas => "Canvas",
    }
}

pub(crate) fn position_is_before_or_equal(left: Position, right: Position) -> bool {
    left.line < right.line || (left.line == right.line && left.column <= right.column)
}

pub(crate) fn detect_document_format(path: &Path, document: &Document) -> DocumentFormat {
    let path_format = DocumentFormat::from_path(path);
    if path_format == DocumentFormat::Canvas {
        return DocumentFormat::Canvas;
    }
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

pub(crate) fn looks_like_markdown_document(document: &Document) -> bool {
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

pub(crate) fn is_markdown_hint(trimmed: &str) -> bool {
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

pub(crate) fn is_markdown_bullet_hint(trimmed: &str) -> bool {
    let mut chars = trimmed.chars();
    let Some(marker) = chars.next() else {
        return false;
    };
    if !matches!(marker, '-' | '*' | '+') {
        return false;
    }
    chars.next().is_some_and(char::is_whitespace)
}

pub(crate) fn is_markdown_ordered_list_hint(trimmed: &str) -> bool {
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

pub(crate) fn is_fountain_hint(trimmed: &str) -> bool {
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
