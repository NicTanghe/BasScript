Basscript Specsheet - Alternative Menus

Goal

Move document metadata editing out of visible rendered page text. Markdown entity files can keep YAML front matter as the durable file format, but the rendered view should not show the raw front matter block. Instead, Basscript should render a compact metadata control area above the first page with dropdown menus and typeable fields for the same values.

Example source front matter

```yaml
---
id: entity_the_unruly_market_001
target: the-unruly-market
type: plot
name: 'The Unruly Market'
aliases: []
status: draft
---
```

The source above remains valid Markdown file data. In rendered Markdown views, the user should see editable controls for `id`, `target`, `type`, `name`, `aliases`, and `status`, not the literal YAML lines.

Epic U - Rendered Metadata Controls

U1. Hide YAML front matter in rendered Markdown

Status: todo

Rules

- Processed/rendered Markdown views must hide a leading YAML front matter block delimited by `---` lines.
- The hidden block includes the opening delimiter, all metadata lines, and the closing delimiter.
- Basscript must only treat a block as front matter when it starts at the beginning of the document, allowing an optional UTF-8 BOM before the first `---`.
- Raw/source views keep showing the literal YAML so the file remains directly editable.
- Split view shows raw YAML in the raw panel and hides it in the processed panel.
- Focus/current-line raw rendering must not leak the full metadata block into normal rendered reading. If the cursor is inside front matter, the raw editable line may be shown only in the raw/source editing surface.
- Hiding front matter must not delete it, change it, or move the document cursor unexpectedly.
- The front matter block must not be replaced with a fake Markdown heading derived from `target`. Titles should come from real body content or from the metadata controls.

Acceptance

- Opening a Markdown entity file with front matter does not show `---`, `id:`, `target:`, `type:`, `name:`, `aliases:`, or `status:` as page text in processed view.
- The first visible rendered page starts with the Markdown body content after the front matter, except for the metadata controls above the page.
- Raw view still shows the exact source front matter.
- Split view keeps the two behaviors separate.
- A file without front matter renders exactly as before.
- Unterminated front matter does not hide the whole document; it shows a recoverable metadata error and leaves the source editable.

U2. Metadata control area above the first page

Status: todo

Rules

- When a rendered Markdown document has valid entity front matter, show a compact metadata control area above page one.
- The control area is part of the editor chrome for the rendered document, not body text on the page.
- The control area scrolls and zooms in a predictable way with the first page header area, but it must not consume page body space or affect pagination.
- The control area appears only once, above the first rendered page, not above every page.
- If the document has no front matter, the control area is hidden unless the user explicitly creates metadata later.
- The area should remain visually quiet and dense enough for repeated editing. Avoid large cards, hero styling, or explanatory in-app text.
- Controls must remain usable in processed mode and split mode.

Acceptance

- A rendered entity Markdown file displays metadata controls above the first page.
- The same controls are not repeated on page two or later.
- Pagination of the Markdown body is unchanged by opening, closing, or editing the metadata controls.
- Zooming the rendered page keeps the controls aligned with the first-page document area.
- Long field values do not overflow the rendered panel.

U3. Field controls and editing behavior

Status: todo

Rules

- `id` is a typeable text field.
- `target` is a typeable target-key field with validation using the existing target-key rules.
- `type` is a combo field: a dropdown of known entity types plus free typing for custom types.
- `name` is a typeable text field.
- `aliases` is an editable list field. It may use chips, rows, or a comma/newline separated input, but it must round-trip to a YAML array.
- `status` is a combo field: a dropdown of known statuses plus free typing for custom status values.
- Dropdown options should be seeded from the story index when available, including custom types/statuses found in the workspace.
- Common type suggestions should include at least `character`, `prop`, `place`, `faction`, `concept`, `scene`, and `plot`.
- Empty `aliases` must display as an empty editable list, not as raw `[]` text.
- Unknown front matter keys should be preserved. They may appear in an expandable "other metadata" editor later, but v1 must not delete them.
- Field edits participate in undo/redo as document edits.
- Field edits mark the document dirty and save back into the front matter.

Acceptance

- Editing `name` in the rendered metadata controls updates `name:` in the source YAML.
- Choosing or typing a new `type` updates `type:` and keeps custom values valid.
- Editing aliases can create `aliases: []` and non-empty alias arrays without corrupting YAML.
- Unknown custom `type` and `status` values remain selectable/typeable.
- Unknown front matter keys survive a load-edit-save round trip.

U4. YAML write-back and formatting preservation

Status: todo

Rules

- YAML front matter remains the source of truth on disk.
- Use a structured front matter parser/writer where practical. Avoid brittle line-only rewrites for arrays, quoting, comments, or unknown keys.
- Preserve existing key order for known fields when possible.
- Preserve unknown keys and comments when possible.
- If exact formatting cannot be preserved, rewrite front matter into a stable canonical format instead of mixing old and new formatting.
- Canonical known-field order is `id`, `target`, `type`, `name`, `aliases`, `status`, followed by unknown preserved keys.
- String values containing spaces, punctuation, leading/trailing whitespace, `:`, `#`, brackets, or quote characters must be quoted safely.
- `aliases` must save as a YAML array. Single-line arrays are acceptable for short lists; block arrays are acceptable if that becomes easier to edit safely.
- Save failures must leave the in-memory document recoverable and show a useful status message.

Acceptance

- Editing controls and saving produces valid YAML front matter.
- Quoted names such as `'The Unruly Market'` remain valid after editing.
- Alias arrays with zero, one, or multiple entries reload correctly.
- Comments or unknown keys are not silently discarded in the normal edit path.
- Invalid write-back does not overwrite the file with broken front matter.

U5. Validation, errors, and indexing integration

Status: todo

Rules

- Required entity metadata fields are `target`, `type`, `name`, and `aliases`.
- `id` and `status` should be displayed when present. Missing optional fields may be empty without blocking body rendering.
- Invalid required fields are shown inline in the metadata control area and in the status line.
- Invalid metadata should not hide the Markdown body.
- The Story Index Database must update after metadata edits are saved or otherwise committed.
- Duplicate targets should be reported clearly and should not be silently auto-fixed.
- Changing `target` should warn when it can break existing links. Automatic link rewrites are out of scope for v1 unless implemented as a separate explicit command.

Acceptance

- Clearing `target` shows validation feedback and prevents committing invalid front matter.
- A duplicate target error identifies the conflicting file when the story index can resolve it.
- After changing `type` or `name`, autocomplete and story queries reflect the new metadata after the index refreshes.
- Broken metadata does not crash rendering, autocomplete, indexing, or save.

U6. Keyboard, focus, and menu behavior

Status: todo

Rules

- Clicking a metadata field focuses that field instead of placing the document cursor in body text.
- `Tab` and `Shift+Tab` move through metadata fields in visual order while focus is inside the metadata area.
- `Enter` commits a single-line field or accepts a dropdown selection.
- `Esc` closes an open dropdown first; if no dropdown is open, it returns focus to the editor body.
- Arrow keys navigate open dropdown rows without moving the document cursor.
- Dropdowns must render above page text and must not be clipped by the rendered panel.
- Normal editor shortcuts such as save keep working while a metadata field is focused, unless the field is actively handling the key.
- Vim mode, command menu, autocomplete menu, and metadata dropdowns must have explicit input priority so keystrokes do not leak into the document.

Acceptance

- Typing in `name` changes the field value, not the Markdown body.
- Opening a `type` dropdown and pressing Down changes the highlighted dropdown row, not the document cursor.
- `Esc` closes a dropdown without changing field text.
- `Ctrl+S` or platform save persists metadata edits.

U7. Layout and visual constraints

Status: todo

Rules

- The metadata area should use compact labels and familiar controls, not prose explaining how the feature works.
- Field labels must remain readable at supported zoom levels.
- Long `id`, `target`, and alias values should truncate, wrap, or scroll inside their own control bounds without overlapping adjacent fields.
- The control area should adapt from a multi-column row on wide panels to stacked rows on narrow panels.
- Dropdown menus must have stable row heights and internal scrolling for long option lists.
- The controls must not overlap the first page, page margin, status line, top menu, or side explorer.

Acceptance

- A narrow rendered panel still shows all metadata fields without text overlap.
- A long alias list remains editable without expanding over body text.
- Dropdown rows do not resize the document layout when the highlighted item changes.

Implementation notes

- Existing Markdown rendering already has a front matter display path. This feature should replace the current rendered-title behavior with explicit metadata controls.
- The core entity front matter model lives around `core/src/links`. Reuse its target validation and entity metadata shape where possible.
- The story index spec defines `target`, `type`, `name`, `aliases`, and `status` as indexed entity metadata. Keep this metadata editor consistent with that source of truth.
- Tests should cover front matter detection with and without a UTF-8 BOM, hiding in processed view, raw view preservation, write-back of each known field, alias array round trips, invalid target validation, and unknown key preservation.
