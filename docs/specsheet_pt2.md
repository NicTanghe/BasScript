Basscript Specsheet Part 2 - Explorer File Management

Goal

Make the left workspace/hamburger explorer usable for normal file management without relying on clunky native save dialogs.

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

- `j` and `k` move the selected row only.
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
- `Save As` remains available for choosing a new path.
- Newly created files become the current save path immediately.
- Status messages distinguish `Saved`, `Save As canceled`, and `Save failed`.

G7. Keybind surface

Status: todo

Explorer focused bindings:

- `j` / `k`: move selection down/up
- `h`: collapse selected folder or move selection to parent
- `l` / `o` / `Enter`: expand/open selected row
- `n`: create file/folder prompt using the selected folder context
- `d`: delete selected row with confirmation
- `r`: rename selected row, optional after create/delete lands
- `R`: refresh workspace tree
- `q` or `Esc`: leave explorer focus

Acceptance

- These bindings are documented in the keybinds UI.
- Bindings only apply while the explorer is focused.
- Existing Ctrl/Cmd-based global shortcuts keep working.

Implementation notes

Current code already has a workspace sidebar, folder/file rows, workspace root persistence, native workspace/save dialogs, and workspace rescanning. The likely implementation areas are:

- `ui/src/pannels/text/explorer.rs` for row selection, active/open document styling, selected folder context, create/delete actions, and refresh.
- `ui/src/editor/dialogs.rs` for separating direct Save from Save As behavior.
- `ui/src/editor/core.rs` for new shortcut actions/state fields.
- `settings/keybinds.ron` for any configurable shortcut additions.
