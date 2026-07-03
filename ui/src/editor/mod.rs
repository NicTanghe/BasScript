// Core types, state, constants, plugin wiring.
include!("core.rs");
// Canvas document state, viewport helpers, and canvas panel components.
include!("canvas.rs");
// Story index database workspace wiring.
include!("story_index.rs");
// Story query sheet overlay and result formatting.
include!("story_query_sheet.rs");
// Status bar formatting and layout.
include!("status_line.rs");
// In-app command menu prompt and command dispatch.
include!("command_menu.rs");
// Link autocomplete trigger context state.
include!("autocomplete.rs");
// Processed pane pagination/cache/styling and text layout helpers.
include!("processed.rs");
// Caret component, blink timer, and caret placement logic.
include!("caret.rs");
// UI hierarchy and toolbar/settings widgets.
include!("ui_setup.rs");
// Draggable panel splitters and pane sizing.
include!("splitters.rs");
// Persistent settings I/O and margin/scale helpers.
include!("settings.rs");
// Internal script linking navigation and click handling.
include!("linking/mod.rs");
// Selection state, pointer behavior, and selection rendering.
include!("selection.rs");
// Text panel-specific logic.
include!("../pannels/text/explorer.rs");
include!("../pannels/text/explorer_actions.rs");
include!("../pannels/text/plain.rs");
include!("../pannels/text/processed.rs");
// Scroll mode input handlers and overlays.
include!("../pannels/text/scrolling/modes/shared.rs");
include!("../pannels/text/scrolling/modes/wheel.rs");
include!("../pannels/text/scrolling/modes/ctrl_left_drag.rs");
include!("../pannels/text/scrolling/modes/middle_autoscroll.rs");
// Native file dialog and shortcut handling.
include!("dialogs.rs");
// System clipboard read helpers for paste commands.
include!("clipboard.rs");
// Optional Vim-style modal editing commands.
include!("vim.rs");
// Text editing/navigation/mouse interaction systems.
include!("editing.rs");
// Rendering systems.
include!("rendering/mod.rs");
