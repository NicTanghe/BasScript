Basscript Specsheet - Linux Copy/Paste and Host Clipboard

Goal

Make copy, cut, paste, and Vim-mode clipboard operations work reliably in the Linux build while sharing the normal host operating system clipboard.

This work should fix the broken Linux behavior without creating a separate Basscript-only clipboard path. Text copied from Basscript should paste into other Linux applications, and text copied from other Linux applications should paste into Basscript.

Existing behavior that should already be implemented

Copy/paste is expected to already exist as part of the editor's normal text-editing behavior. This task should first verify the current implementation path before adding new behavior.

Expected existing surfaces

- `Ctrl+C` copies the current editor selection.
- `Ctrl+X` cuts the current editor selection when the active surface is editable.
- `Ctrl+V` pastes plain text at the editor cursor.
- Menu or toolbar copy/paste commands, if present, dispatch to the same editor operations as the keyboard shortcuts.
- Prompt inputs, path fields, and other editable UI text fields use the same host clipboard integration where practical.
- Non-Linux behavior should remain unchanged.

Investigation requirement

- Determine whether Linux copy/paste is failing because of shortcut routing, focus handling, editor selection state, clipboard API integration, or platform build configuration.
- Prefer repairing the existing editor command path over adding duplicate Linux-only command handling.
- Confirm whether the failure affects the main editor, Vim mode, explorer prompts, command prompts, or all editable text surfaces.

Epic M - Linux Clipboard Integration

M1. Host clipboard adapter

Status: todo

Use the project or framework clipboard API to read and write the host OS clipboard on Linux.

Rules

- Plain text is the required v1 clipboard format.
- Clipboard writes must be readable by external Linux applications.
- Clipboard reads must accept plain text copied from external Linux applications.
- Use the standard desktop clipboard selection, commonly `CLIPBOARD`, rather than relying on the X11 primary selection.
- Support both X11 and Wayland when the underlying framework allows it.
- Avoid shelling out to `xclip`, `xsel`, `wl-copy`, or `wl-paste` for normal runtime behavior.
- Clipboard access must not block the UI thread for noticeable time.
- Unicode text must be preserved.
- Empty clipboard reads fail gracefully with a status message or no-op.

Acceptance

- Copy text in Basscript and paste it into an external editor on Linux.
- Copy text from an external Linux editor and paste it into Basscript.
- Unicode text survives a copy/paste round trip.
- Empty selections do not corrupt the clipboard.
- Clipboard failures do not crash the app.

M2. Editor shortcut routing

Status: todo

Route copy, cut, and paste shortcuts through the active editable surface.

Rules

- `Ctrl+C`, `Ctrl+X`, and `Ctrl+V` operate on the focused editor or focused text prompt.
- Shortcuts must not fire against the main document while the explorer, path prompt, command prompt, or modal dialog owns text focus.
- Copy without a selection should be a no-op unless the active surface has an established line-copy behavior.
- Cut without a selection should be a no-op unless the active surface has an established line-cut behavior.
- Paste inserts at the active cursor and participates in undo history as one paste edit.
- Pasted text containing newlines must preserve line breaks.

Acceptance

- Main editor copy, cut, and paste work with keyboard shortcuts.
- In-app prompts can paste text without also modifying the document behind them.
- Copy/cut from one Basscript surface can paste into another Basscript surface through the same host clipboard.
- Paste creates a single undoable edit.

M3. Selection and focus correctness

Status: todo

Make copy/cut depend on the actual active selection and make paste depend on the actual active cursor.

Rules

- The editor must have one authoritative focused editing target.
- Selection state must remain valid when focus changes between editor panels, explorer prompts, and modal inputs.
- Copy/cut should use the visible selected text in the active panel.
- Paste should insert into the active document only when the active surface is editable.
- Read-only processed views may allow copy but must not allow paste.

Acceptance

- Selecting text in the raw editor and pressing `Ctrl+C` copies that selected text.
- Selecting text in a prompt and pressing `Ctrl+C` copies the prompt selection, not the document selection.
- Pasting while a read-only processed view is focused does not mutate the document unless the app intentionally redirects focus to an editable raw source location.

Epic N - Vim Mode Host Clipboard

N1. Shared host clipboard for Vim mode

Status: todo

The Vim-style editing mode must share the host copy/paste buffer. It must not keep yanks and puts in a private Vim-only clipboard in v1.

Rules

- The default Basscript Vim register maps to the host OS clipboard.
- Vim yank operations write plain text to the host clipboard.
- Vim put/paste operations read plain text from the host clipboard.
- Vim delete/change operations that behave as cuts should update the host clipboard when they replace or remove selected text.
- Visual-mode yank and paste use the same host clipboard as `Ctrl+C` and `Ctrl+V`.
- Insert-mode `Ctrl+C`, `Ctrl+X`, and `Ctrl+V` keep the same behavior as non-Vim editing where those shortcuts are enabled.
- Normal-mode printable command keys must still not insert literal text into the document.
- Explorer focus rules take priority over Vim clipboard bindings.

Suggested v1 Vim commands

- `y` on an active selection copies the selected text to the host clipboard.
- `yy` copies the current logical line to the host clipboard.
- `p` pastes host clipboard text after the cursor or current line, depending on whether the clipboard text is linewise.
- `P` pastes host clipboard text before the cursor or current line, depending on whether the clipboard text is linewise.
- `d` on an active selection cuts the selected text to the host clipboard.
- `dd` cuts the current logical line to the host clipboard.

Acceptance

- Yank text in Vim mode, then paste it into an external Linux application.
- Copy text from an external Linux application, then paste it with Vim `p`.
- `Ctrl+C` and Vim `y` write to the same host clipboard.
- `Ctrl+V` and Vim `p` read from the same host clipboard.
- No separate Vim-only clipboard is required to move text between Basscript and the host desktop.

N2. Vim mode linewise vs characterwise paste

Status: todo

Track enough metadata for natural Vim-style pasting without breaking host clipboard interoperability.

Rules

- The host clipboard stores plain text only.
- Basscript may keep transient in-app metadata describing whether the most recent Vim yank/cut was linewise or characterwise.
- That metadata must be optional; pasted host text from another app should still work without it.
- If clipboard text ends with a newline and no metadata exists, Basscript may treat it as linewise.
- If clipboard text has no trailing newline and no metadata exists, Basscript should treat it as characterwise.

Acceptance

- `yy` followed by `p` inserts a full line in a natural Vim-style location.
- Character selection yank followed by `p` inserts at the character cursor.
- Text copied from another app pastes predictably even without Basscript metadata.

Epic O - Linux Verification

O1. Manual Linux clipboard matrix

Status: todo

Verify behavior in an actual Linux desktop session.

Acceptance

- Basscript to external app copy works.
- External app to Basscript paste works.
- Basscript Vim yank to external app paste works.
- External app copy to Basscript Vim paste works.
- Main editor shortcuts work.
- Prompt input shortcuts work.
- Empty selection and empty clipboard cases are harmless.

O2. Automated coverage where practical

Status: todo

Add focused tests for command dispatch and editor mutations without depending on a live desktop clipboard when CI cannot provide one.

Rules

- Unit-test editor copy/cut/paste command behavior using an injectable clipboard interface.
- Unit-test Vim clipboard command behavior using the same injectable interface.
- Keep platform clipboard integration thin enough that it can be manually tested on Linux and mocked in automated tests.

Acceptance

- Tests prove copy/cut write the expected plain text.
- Tests prove paste inserts the clipboard text at the expected cursor/selection.
- Tests prove Vim and non-Vim paths use the same clipboard interface.
- Existing editing and parser tests still pass.

Out of scope for v1

- Rich text clipboard formats.
- Image clipboard support.
- Clipboard history.
- X11 primary-selection mouse paste.
- Remote SSH clipboard forwarding.
- Full Vim register compatibility beyond the default host-backed clipboard behavior.
