# 0.19 port

## Purpose

Port BasScript to Bevy 0.19 while keeping the current document editor's
screenplay, Markdown, Canvas, Vim, selection, and undo behavior intact.

This sheet records Bevy 0.19 capabilities that are good candidates for a
separate, deliberately tested reduction of local UI code and dependencies. It
is not part of the compatibility port itself.

## Guardrails

- Preserve the rich document editor and its source-to-rendered-text mapping.
- Keep deterministic bundled fonts for document layout unless a replacement
  has been visually verified on Linux, Windows, and macOS.
- Retain application-level undo/redo and Vim behavior.
- Make each replacement independently reversible and testable.

## Candidates

### Simple text controls: adopt `EditableText`

**Targets:** `ui/src/editor/command_menu.rs`,
`ui/src/pannels/text/explorer_actions.rs`, and (after a small prototype)
simple scalar fields in `ui/src/editor/metadata_controls.rs`.

**Proposal:** replace the handwritten keyboard-input, cursor, selection, and
paste loops with Bevy 0.19's `EditableText` for these single-line controls.
Keep command parsing, prompt actions, validation, and application focus rules
in BasScript.

**Why:** native controls provide cursor movement, selection, word navigation,
IME, clipboard integration, blinking cursor, and horizontal scrolling. This
removes isolated duplicated input logic without changing the editor model.

**Acceptance criteria:** Enter/Escape behavior remains unchanged; a focused
control does not leak input to the document editor; copy/paste, Unicode input,
and a mouse selection work on all supported desktop platforms.

**References:** [Bevy 0.19 text input release notes](https://bevy.org/news/bevy-0-19/#text-input),
[EditableText API](https://docs.rs/bevy/0.19.0/bevy/ui/widget/struct.EditableText.html).

### Canvas plain-text editing: `EditableText` feasibility spike

**Targets:** `ui/src/editor/canvas.rs` and
`ui/src/editor/rendering/canvas.rs` in the `PlainInteractive` path only.

**Proposal:** prototype a native editable text node for a plain Canvas text
card. Continue rendering the rich Markdown/Fountain preview separately.

**Why:** this path currently owns custom caret geometry, selection rectangles,
hit testing, and text entry. It is the largest plausible code-reduction target.

**Risks / non-goals:** do not replace the main raw or processed editor. The
native control is plain text and currently lacks undo/redo; Canvas integration
must retain Vim bindings, app history, zoom, drag/focus behavior, and model
synchronization before it can replace the custom path.

**Acceptance criteria:** a prototype preserves Canvas node undo/redo,
selection, Unicode, wrapping, mouse placement, zoom, and navigation before
any existing geometry is removed.

### Font-source and variable-font consolidation

**Targets:** `ui/src/editor/core.rs` font resources and Markdown text styling.

**Proposal:** evaluate Bevy 0.19 `FontSource`, `FontWeight`, `FontStyle`, and
`FontWidth` for `SegoeUIVF.ttf`. The four Markdown font resource entries
currently refer to the same variable-font file.

**Why:** one font source plus native variation properties may remove duplicate
routing and make styles clearer.

**Risks:** verify that the bundled font has the desired italic and weight axes;
keep Courier Prime's explicit static faces unless output metrics match.

**Acceptance criteria:** screenshot/metric comparison of Markdown and
screenplay pagination at supported zoom levels before deleting a font handle
or asset.

**References:** [Bevy 0.19 richer text release notes](https://bevy.org/news/bevy-0-19/#richer-text),
[FontSource API](https://docs.rs/bevy/0.19.0/bevy/text/enum.FontSource.html).

### System-font packaging experiment

**Targets:** `fonts/` assets and the Bevy feature list.

**Proposal:** optionally offer a build/profile that uses `FontSource::SystemUi`
and `FontSource::Monospace` with system font discovery, instead of bundling
editor-chrome fonts.

**Why:** the font tree is roughly 2.2 MB, mostly `SegoeUIVF.ttf`.

**Risks:** font metrics vary by OS and directly affect screenplay columns and
pagination. This must remain opt-in until a stable cross-platform policy is
chosen.

### Clipboard consolidation for native controls

**Targets:** `ui/src/editor/clipboard.rs` and target-specific `arboard`
configuration in `ui/Cargo.toml`.

**Proposal:** enable Bevy's `system_clipboard` feature when simple controls
migrate to `EditableText`, then assess whether their direct `arboard` wrapper
can go away.

**Non-goal:** retain the current clipboard adapter while the rich document
editor and Vim registers rely on it.

### Settings framework evaluation

**Targets:** custom RON persistence in `ui/src/editor/settings.rs` and
`ui/src/editor/core.rs`.

**Proposal:** compare Bevy 0.19's settings framework with the current RON
format in a separate migration.

**Risks:** preserve existing user settings, migration behavior, and editor
layout compatibility. Do not replace the current format as part of the Bevy
compatibility port.

### Dependency / vendor cleanup — completed

**Targets:** the former `vendor/wgpu-hal` and `vendor/bevy_winit` overrides.

**Completed:** the port uses upstream Bevy 0.19 / wgpu 29 rather than the old
0.18 / wgpu 27 patches. The resize-constraint behavior now lives in
application-owned code, and the obsolete vendor sources were removed.

**Follow-up verification:** smoke-test Linux, Windows, and macOS window
creation, glass surfaces, resize constraints, Canvas selection, and rendering
without a crates.io override.

## Explicitly retain custom implementation

The main raw/processed editor remains custom. It combines Fountain/Markdown
parsing, rich rendering, document-to-display mapping, page layout, links,
Vim, application undo/redo, Canvas behavior, and editor-specific selection.
Bevy's `EditableText` is a useful plain-text building block, not a replacement
for this document model.
