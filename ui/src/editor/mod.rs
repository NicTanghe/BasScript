// Core types, state, constants, plugin wiring.
include!("core.rs");
// Processed pane pagination/cache/styling and text layout helpers.
include!("processed.rs");
// UI hierarchy and toolbar/settings widgets.
include!("ui_setup.rs");
// Persistent settings I/O and margin/scale helpers.
include!("settings.rs");
// Native file dialog and shortcut handling.
include!("dialogs.rs");
// Text editing/navigation/mouse interaction systems.
include!("editing.rs");
// Rendering + caret systems.
include!("render.rs");
