Basscript Specsheet Part 2 - Explorer, Vim Mode, and Commands

Goal

Make the left workspace/hamburger explorer usable for normal file management without relying on clunky native save dialogs. Add an optional basic Vim-style editing mode before adding the in-app command window, so command input can be routed through the final modal editing model.

Reference behavior

LunarVim exposes its side file explorer through `<leader>e`, where leader is Space by default. Inside the explorer, `g?` shows available mappings. LunarVim uses nvim-tree, whose user-facing defaults include `a` for create file/directory, `d` for delete, `r` for rename, `R` for refresh, `o`/Enter for open, `-` for parent directory, and `q` for close.

Basscript should borrow the modal explorer idea and single-key file operations, but use `n` for new file/folder because that is the requested project behavior.

References

- LunarVim keybinds overview: https://www.lunarvim.org/docs/beginners-guide/keybinds-overview
- LunarVim keybindings configuration: https://www.lunarvim.org/docs/configuration/keybindings
- nvim-tree docs: https://github.com/nvim-tree/nvim-tree.lua

Epic G - Workspace Explorer File Management

G1. Explorer focus mode

Status: todo

The explorer must have an active/focused state separate from text editing.

Acceptance

- Clicking any folder or file row focuses the explorer.
- Explorer-only bindings do not fire while the editor text area is focused.
- `Esc` returns focus to the editor.
- The focused explorer row is visibly distinct.
- Opening/toggling the explorer should make it possible to focus it without using the mouse later.

G2. Selection vs active document

Status: todo

The explorer needs two separate concepts:

- Selected row: the row currently targeted by explorer keyboard commands.
- Active/open document: the file currently loaded and visible in the editor viewport.

These must not be used interchangeably.

Rules

- `j`, `k`, Up, and Down move the selected row only.
- Holding `j`, `k`, Up, or Down for the normal repeat delay, approximately 0.1 seconds, keeps moving the selected row.
- Moving the selected row never opens a file by itself.
- Pressing `Enter`, `o`, or `l` on a selected file opens it and makes it the active/open document.
- Clicking a file selects that row and opens it.
- Clicking a folder selects that row and toggles or focuses that folder.
- The active/open document stays visibly marked even when the selected explorer row moves elsewhere.
- New file/folder and delete actions operate on the selected row, not necessarily on the active/open document.

Acceptance

- The user can move selection through files with nvim-style keys without changing the editor viewport.
- The editor viewport changes only after a click or an explicit open command.
- The selected row and active/open file can be different at the same time.
- Visual styling makes that difference clear.

G3. Selected folder context

Status: todo

Track the folder context from the selected explorer row.

Rules

- If the selected row is a folder, the selected folder context is that folder.
- If the selected row is a file, the selected folder context is that file's parent folder.
- If no row is selected, use the workspace root.
- If no workspace root exists, the new/delete actions are disabled with a status message.

Acceptance

- The selected folder context updates when keyboard selection moves.
- The selected folder context updates when rows are clicked.
- Create prompts show the selected folder context as a full absolute path.

G4. `n` create file/folder prompt

Status: todo

When the explorer is focused and `n` is pressed, show an in-app path prompt.

Prompt behavior

- The input is prefilled with the full absolute path of the selected folder context plus the platform separator.
- The cursor starts at the end so the user can type the new name immediately.
- If the final submitted path ends in `/` or `\`, create a folder.
- Otherwise create a file.
- Do not auto-add an extension in v1. The typed path is the source of truth.
- `Enter` confirms.
- `Esc` cancels.
- Empty suffix after the selected folder context is invalid.

Acceptance

- Creating `C:\root\scene.fountain` creates a file.
- Creating `C:\root\notes\` creates a folder.
- Parent folders are created as needed.
- After creation, the explorer refreshes.
- New files that match explorer-supported extensions appear in the tree.
- Newly created files open immediately and become the active/open document and current save path.
- Newly created folders expand and become the selected row or selected folder context.

G5. `d` delete prompt

Status: todo

When the explorer is focused and `d` is pressed, prompt before deleting the selected row.

Acceptance

- Files and folders can be deleted from the explorer.
- `Enter` confirms deletion from the delete prompt.
- `n` or `Esc` cancels deletion from the delete prompt.
- Delete acts on the selected row, even if a different file is active/open in the editor viewport.
- Folder deletion requires explicit confirmation and shows that the folder is non-empty when applicable.
- Deleting the currently open file clears or redirects the editor state safely instead of leaving a stale path.
- After deletion, the explorer refreshes and selection moves to a nearby valid row.
- Delete failures show a useful status message.

G6. Save behavior cleanup

Status: todo

Make saving predictable after explorer-created files.

Acceptance

- `Save` writes directly to the current save path without opening a dialog.
- `Ctrl+S` writes directly to the current save path without opening a dialog.
- `Save As` remains available for choosing a new path.
- Newly created files become the current save path immediately.
- Status messages distinguish `Saved`, `Save As canceled`, and `Save failed`.

Epic H - Optional Basic Vim Mode

H1. Vim mode setting

Status: todo

Add a settings-menu toggle for Vim mode.

Rules

- Vim mode is off by default unless the user enables it.
- The toggle persists across app launches.
- When Vim mode is off, existing editor keyboard behavior remains unchanged.
- When Vim mode is on, text editing is modal: Normal mode for commands and Insert mode for text input.
- The status line or editor chrome shows the active Vim mode.

Acceptance

- The user can enable and disable Vim mode from the settings menu.
- Toggling Vim mode does not lose the current document, cursor, selection, or scroll position.
- Disabling Vim mode immediately restores normal text-entry behavior.

H2. Normal and Insert modes

Status: todo

Rules

- `Esc` enters Normal mode from Insert mode, Visual mode, prompts, or active selections where applicable.
- `i` enters Insert mode from Normal mode at the current cursor.
- In Insert mode, typing, deletion, newline insertion, and existing shortcuts behave like the non-Vim editor.
- In Normal mode, printable command keys must not insert text into the document.
- Mouse clicks still place the cursor in the selected editor panel; Vim mode remains active.
- Existing explorer focus rules take priority when the explorer is focused.
- Normal mode uses a wide/block-style caret so it is visually distinct from Insert mode.

Acceptance

- Pressing `i` in Normal mode allows text entry.
- Pressing `Esc` from Insert mode stops text entry and returns to Normal mode.
- Normal-mode commands never leak command characters into the document.
- The caret clearly changes shape between Normal mode and Insert mode.

H3. Normal-mode movement

Status: todo

Rules

- Support the standard `h` / `j` / `k` / `l` directional cluster:
  - `h`: move left
  - `j`: move down
  - `k`: move up
  - `l`: move right
- Arrow keys continue to move the cursor.
- Repeating held movement keys should use the same repeat behavior as arrow navigation.
- Vertical movement follows the last selected editor panel:
  - processed/formatted panel: move by visible rendered lines, including soft wraps
  - raw/plain panel: move by raw logical document lines
- Horizontal movement moves by document character columns.

Acceptance

- The user can navigate without leaving Normal mode.
- Movement respects processed visual wrapping when the processed panel was last selected.
- Movement in the raw/plain panel keeps the existing logical-line behavior.

H4. Visual selection

Status: todo

Rules

- `v` enters characterwise Visual mode from Normal mode.
- `V` enters linewise Visual mode from Normal mode.
- Movement keys extend the active Visual selection.
- `Esc` exits Visual mode and clears the active Visual selection.
- Mouse selection and shift-selection continue to work while Vim mode is enabled.

Acceptance

- Characterwise Visual mode can select part of a line or span multiple lines.
- Linewise Visual mode selects whole logical document lines.
- The rendered selection is visible in both raw and processed panels.

H5. Yank, paste, and delete basics

Status: todo

Rules

- Maintain an internal Vim register for yanked/deleted text.
- `yy` yanks the current logical document line as a linewise register.
- `y` yanks the active Visual selection and returns to Normal mode.
- `p` pastes from the register.
- Linewise paste inserts below the current logical line.
- Characterwise paste inserts at or after the current cursor using Vim-like behavior.
- `dd` deletes the current logical document line into the register.
- `d` deletes the active Visual selection into the register and returns to Normal mode.
- Register behavior should not depend on native clipboard availability in v1.

Acceptance

- `yy` followed by `p` duplicates the current line below it.
- Visual selection followed by `y` and `p` copies and pastes the selected text.
- `dd` removes the current line and allows pasting it back with `p`.
- Paste participates in undo history as a normal edit.

H6. Vim-mode key handling boundaries

Status: todo

Rules

- Vim normal/visual commands apply only when editor text focus is active.
- Explorer focus, path prompts, delete prompts, settings screens, and future command windows consume their own input first.
- Ctrl/Cmd global shortcuts keep working unless a shortcut is explicitly changed in keybind settings.
- Vim mode should not break configurable keybinds; conflicting bindings are rejected or clearly reported.

Acceptance

- Vim commands do not trigger while typing into prompts or settings fields.
- Global save/open/view shortcuts still work with Vim mode enabled.
- The keybinds UI documents Vim mode bindings and any conflicts.

Epic I - Command Menu

I1. Command menu

Status: todo

Add an in-app command menu for file workflow commands after Vim mode lands, so command input respects Normal/Insert/Visual routing.

Rules

- Pressing `<Space>:` opens the command menu from the editor.
- When Vim mode is enabled, pressing `:` in Normal mode also opens the command menu.
- The command menu accepts short text commands.
- The first registered command must be `w`, short for write.
- Running `w` saves the current file to the current save path, matching `Ctrl+S` and direct `Save` behavior.
- Running `w` must not open a native save dialog.
- If the current document has no save path, `w` should use the same fallback behavior as direct `Save`.

Acceptance

- Pressing `<Space>:` opens the command menu from the editor without changing the document.
- In Vim Normal mode, pressing `:` opens the command menu without inserting `:`.
- Typing `w` and confirming saves the current file.
- `w`, `Ctrl+S`, and `Save` share the same save implementation and status messages.
- Command failures show a useful status message.

J1. Keybind surface

Status: todo

Explorer focused bindings:

- `j` / `k`: move selection down/up, repeating while held
- `Up` / `Down`: move selection up/down, repeating while held
- `h`: collapse selected folder or move selection to parent
- `l` / `o` / `Enter`: expand/open selected row
- `n`: create file/folder prompt using the selected folder context
- `d`: delete selected row with confirmation
- `r`: rename selected row, optional after create/delete lands
- `R`: refresh workspace tree
- `q` or `Esc`: leave explorer focus

Global/editor bindings:

- `Ctrl+S`: save the current file to the current save path
- `<Space>:`: open the command menu

Vim mode bindings, when enabled and editor text focus is active:

- `Esc`: Normal mode / leave Visual mode
- `i`: Insert mode
- `h` / `j` / `k` / `l`: left / down / up / right
- Arrow keys: cursor movement
- `v`: characterwise Visual mode
- `V`: linewise Visual mode
- `yy`: yank current line
- `y`: yank active Visual selection
- `p`: paste register
- `dd`: delete current line into register
- `d`: delete active Visual selection into register
- `:`: open command menu from Normal mode

Command menu entries:

- `w`: write/save the current file to the current save path

Acceptance

- These bindings are documented in the keybinds UI.
- Explorer bindings only apply while the explorer is focused.
- Vim mode bindings only apply while Vim mode is enabled and editor text focus is active.
- Existing Ctrl/Cmd-based global shortcuts keep working.

Implementation notes

Current code already has a workspace sidebar, folder/file rows, workspace root persistence, native workspace/save dialogs, and workspace rescanning. The likely implementation areas are:

- `ui/src/pannels/text/explorer.rs` for row selection, active/open document styling, selected folder context, create/delete actions, and refresh.
- `ui/src/editor/dialogs.rs` for separating direct Save from Save As behavior.
- `ui/src/editor/core.rs` for new shortcut actions, Vim mode state, modal input state fields, and command menu state.
- `ui/src/editor/vim.rs` for Vim Normal/Insert/Visual input routing, movement, selection, yank/delete/paste, and register behavior.
- `ui/src/editor/command_menu.rs` for command-window UI, command input, and command dispatch.
- `ui/src/editor/editing.rs` for non-Vim insert-mode text editing and shared cursor movement helpers.
- `settings/keybinds.ron` for any configurable shortcut additions.
