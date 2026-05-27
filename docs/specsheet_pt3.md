Basscript Specsheet Part 3 - Images and Canvas Reference View

Goal

First make image embeds work in processed Markdown files and Fountain scripts using standard Markdown image notation. Then add an Obsidian Canvas / PureRef-like reference board view for arranging images, notes, script files, links, and groups in a spatial workspace.

The image work is intentionally first because the canvas view should reuse the same image path resolver, asset cache, missing-image handling, and preview renderer instead of creating a second image pipeline.

Reference behavior

Markdown image notation:

- `![alt text](relative/or/absolute/path.png)`
- `![alt text](path/to/image.jpg "optional title")`

Obsidian Canvas stores `.canvas` files using the open JSON Canvas format. JSON Canvas 1.0 defines top-level `nodes` and `edges`, node types `text`, `file`, `link`, and `group`, integer `x`/`y`/`width`/`height` coordinates, and array order as z-index.

PureRef is the reference-board inspiration for fast pan/zoom navigation, image-first spatial layout, dragging, scaling, arranging, and using the board as a visual reference surface.

Basscript should borrow the local-file, infinite-canvas, image-board workflow. Remote image URLs are supported when they are direct image targets. Full Obsidian feature parity, PureRef file compatibility, web page thumbnails, video playback, and plugin systems are out of scope for v1.

References

- JSON Canvas spec 1.0: https://jsoncanvas.org/spec/1.0
- JSON Canvas overview: https://jsoncanvas.org/
- Obsidian Canvas help: https://obsidian.md/help/plugins/canvas
- PureRef canvas handbook: https://www.pureref.com/handbook/2.1/canvas/
- PureRef features handbook: https://www.pureref.com/handbook/features/

Epic K - Markdown and Fountain Image Embeds

K1. Shared image embed parser

Status: implemented

Add a core image-embed representation that can be populated by both Markdown and Fountain parsing without depending on Bevy or UI asset handles.

Suggested core model

- `ImageEmbed`
  - `raw_start_column`
  - `raw_end_column`
  - `alt`
  - `target`
  - `title`
- `ParsedLine.image_embeds: Vec<ImageEmbed>`

Rules

- Recognize Markdown image syntax beginning with `![`.
- Support `![alt](target)` and `![alt](target "title")`.
- Preserve the typed target string in core; path resolution belongs in UI/application code because it depends on the current document path and workspace root.
- Image syntax must not be extracted as a normal script link.
- Do not extract image embeds from Markdown fenced code block content.
- Do not require Fountain-specific syntax. Fountain uses the exact same Markdown image notation.
- v1 may reject nested brackets in alt text and complicated escaped destinations, but it must fail gracefully by treating the line as normal text.
- Remote URLs, data URLs, animated image behavior, and captions are not required in v1.

Acceptance

- Markdown parser tests prove `![door](refs/door.png)` produces one image embed.
- Fountain parser tests prove the same notation produces one image embed.
- Existing `[label](target)` script links still work.
- `![alt](target)` does not create a script link.
- Image-like text inside Markdown fenced code does not create an image embed.
- Invalid image syntax leaves the raw line editable and renderable as text.

K2. Image path resolution

Status: implemented v1

Resolve image targets consistently for Markdown files, Fountain scripts, and future canvas file nodes.

Rules

- Relative paths resolve from the current document's save path parent when available.
- If the current document has no save path, relative paths resolve from the current workspace root.
- Absolute local paths are allowed when the platform accepts them.
- Workspace-relative paths are allowed when the target is not found beside the current document.
- `.` and `..` segments are normalized before loading.
- Missing, unreadable, unsupported, or oversized images render as an inline placeholder instead of crashing or blocking editing.
- v1 local supported formats should include at least `png`, `jpg`, `jpeg`, `webp`, and `bmp` if supported by the Bevy image pipeline.
- `svg` may use the existing `resvg` rasterization path if practical; otherwise show an unsupported-image placeholder in v1.
- Animated `gif` is out of scope for v1. A later pass can decode a first frame or add animation.

Acceptance

- `![alt](image.png)` next to `notes.md` loads `notes-folder/image.png`.
- The same syntax in `scene.fountain` resolves relative to `scene.fountain`.
- An unsaved document can still load workspace-relative images.
- A missing image shows alt text and the unresolved target in a stable placeholder.
- Loading failure is cached for a short interval or until the document changes, so the renderer does not retry every frame.

K3. Processed-view image layout

Status: implemented v1

Render image embeds in processed views while keeping raw text editing unchanged.

Rules

- Plain/raw panels always show the literal Markdown image notation.
- Processed panels hide the image syntax and draw image blocks.
- The current raw line in focus mode still shows the literal syntax so the user can edit it.
- Image-only lines render as image blocks.
- Lines with surrounding text render the text and image blocks in source order. v1 may stack them vertically instead of doing true inline image flow.
- Image blocks use the image's natural aspect ratio.
- Image block width is capped to the processed text area.
- Image block height is capped to a reasonable page fraction, with a placeholder status if the image is too large to display naturally.
- Image blocks reserve vertical space in wrapping, pagination, scrolling, caret placement, and selection hit-testing.
- Clicking an image block places the cursor on the source line. v1 does not need per-pixel image editing.
- Selection over an image line highlights the source line or image block consistently.

Acceptance

- A Markdown file containing `![door](door.png)` shows the image in processed mode.
- A Fountain file containing `![door](door.png)` shows the image in processed mode.
- Raw mode and the current raw focus line keep showing editable `![door](door.png)` text.
- Images do not overlap following text, page margins, paper bounds, or the caret.
- Scrolling, zooming, and page layout remain stable when an image finishes loading asynchronously.

K4. Fountain-specific image behavior

Status: implemented

Make image embeds feel natural inside Fountain scripts without damaging screenplay classification.

Rules

- A Fountain line containing only image syntax is treated as an action-like visual block.
- A Fountain image-only line must reset dialogue context like an Action line, so the following line is not accidentally classified as Dialogue because of an earlier Character cue.
- Inline image embeds inside Action or Dialogue lines keep the underlying Fountain line kind for styling and editing, but the image token renders as an image block in processed mode.
- Image embeds must not be uppercased by Character, Scene Heading, or Transition processing.
- Fountain page-break handling must account for reserved image block height.

Acceptance

- `SARAH`, dialogue, image-only line, then another action line classifies correctly.
- An image-only line after a Character cue does not become Dialogue.
- A Transition line that happens to contain image-like text does not crash or uppercase the image path.
- Fountain processed pagination reserves enough space for the rendered image block.

K5. Image asset cache and lifecycle

Status: implemented v1

Add one reusable image asset layer for processed Markdown, processed Fountain, and canvas nodes.

Rules

- Cache by resolved absolute path plus file metadata where practical.
- Do not decode or reload images every frame.
- Cache load failures separately from successful images.
- Reload images when the file changes or after an explicit refresh.
- Store UI asset handles in UI resources, not core parser output.
- Large images should be downscaled for preview rendering when needed.
- Failed image loads must not mark the document dirty.

Acceptance

- Repeated embeds of the same image reuse the same loaded asset.
- Editing text near an image does not reload the image.
- Reopening a document can reuse cached image handles during the app session.
- Deleting or renaming an image file degrades to a placeholder after refresh.

K6. Image insertion and file workflow

Status: todo

Provide a minimal way to insert image notation after rendering support is stable.

Rules

- Dragging an image file from the explorer into a text document inserts `![filename](relative/path)` at the cursor.
- A command-menu action may insert an image by prompting for a path.
- Prefer relative paths when the target is under the current document folder or workspace root.
- Preserve spaces in filenames using normal Markdown destination handling.
- Inserted syntax participates in undo history as a normal text edit.

Acceptance

- Dragging `refs/door.png` into `scene.fountain` inserts usable Markdown image syntax.
- The inserted image renders without manual path editing.
- Unsupported dragged files show a status message and do not change the document.

Epic L - Canvas Data Model and File Format

L1. Canvas document format

Status: implemented v1

Add `.canvas` as a supported document type, using JSON Canvas 1.0 as the storage format.

Rules

- `DocumentFormat` should grow a Canvas-like variant or the app should introduce a separate canvas document path that does not pretend canvas files are line text.
- `.canvas` files load as JSON Canvas, not Fountain.
- A canvas document contains nodes, edges, selection state, dirty state, and a viewport transform.
- Viewport transform should be stored in app state/settings in v1 unless a compatible extension strategy is chosen.
- Unknown JSON fields should be preserved on load/save where practical to avoid destroying data created by other tools.
- Canvas edits that update known fields must preserve unrelated node, edge, and top-level fields by editing the parsed JSON object instead of rebuilding the document from the reduced runtime model.
- A UTF-8 BOM at the start of a `.canvas` file is ignored before parsing.
- Blank or whitespace-only `.canvas` files open as an empty board.
- Invalid JSON or invalid canvas shape opens a readable error state and does not overwrite the file.

Acceptance

- Opening `board.canvas` from the explorer enters canvas view.
- Saving `board.canvas` writes valid JSON Canvas.
- A non-empty `.canvas` file with a UTF-8 BOM opens correctly.
- A blank `.canvas` file opens as an empty board instead of showing an invalid JSON error.
- Moving or editing a node preserves unrelated JSON Canvas fields.
- Invalid `.canvas` files show a recoverable error instead of crashing.
- Opening a `.fountain` or `.md` file still uses the existing editor views.

L2. Canvas node types

Status: implemented v1 rendering; text nodes and node positions are editable

Support the JSON Canvas node types needed for useful reference boards.

Node types

- `text`: editable plain text with Markdown preview styling where possible.
- `file`: local or direct remote file reference. Images render as image previews; Markdown and Fountain files render as note/script preview cards; unknown files render as file placeholders.
- `link`: URL reference with a title/URL placeholder in v1.
- `group`: visual container with optional label and optional background image.

Rules

- All nodes have `id`, `type`, `x`, `y`, `width`, and `height`.
- Node IDs and edge IDs share a uniqueness namespace.
- Node order in the `nodes` array is the z-index order.
- Coordinates and dimensions are stored as integer pixels for JSON Canvas compatibility.
- File node paths use the same resolver and image cache as Markdown/Fountain embeds.
- Image targets may be local paths or direct `http://` / `https://` image URLs.
- A text node containing a Markdown image embed or an HTML `<img src=...>` tag may render the first image target as an image card preview when the node is not being actively edited.
- HTML image tags may span lines and may use quoted or unquoted `src` attributes.

Acceptance

- Text, image file, non-image file, link, and group nodes load from a valid `.canvas`.
- A canvas containing an image file node displays the image.
- A text node containing `![alt](image.jpg)` displays the image preview when not being edited.
- A text node containing `<img src=https://example/image.jpg>` displays the image preview when not being edited.
- A direct remote image URL renders when the Bevy asset loader can fetch it, otherwise a placeholder remains visible.
- Z-order is stable after load/save.
- Missing file nodes show a clear placeholder without deleting the node.

L3. Canvas edge model

Status: implemented v1 basic rendering

Support basic JSON Canvas edges for connecting nodes.

Rules

- Edges contain `id`, `fromNode`, `toNode`, optional sides, optional endpoint styles, optional color, and optional label.
- Dangling edges should remain in the data model but render as invalid or hidden with a status warning.
- v1 supports straight or simple orthogonal connectors. Complex routing is optional.
- Edge labels render near the connector midpoint.

Acceptance

- A valid edge renders between two nodes.
- Moving either node updates the edge endpoint.
- Saving preserves edge labels, colors, sides, and endpoint settings.
- A dangling edge does not crash rendering or saving.

Epic M - Canvas View Shell and Navigation

M1. Canvas view mode

Status: implemented v1

Add a dedicated canvas view for `.canvas` files.

Rules

- Explorer opening rules route `.canvas` files into canvas view.
- Canvas view has its own focus mode separate from text editing and explorer focus.
- Text editor shortcuts do not type into canvas unless a text node editor is active.
- Normal document text input is disabled while a canvas is active unless the active input target is a canvas text node.
- Existing global save/open/settings shortcuts keep working.
- Switching from a canvas file to a text file restores the normal editor layout.

Acceptance

- Clicking a `.canvas` file opens a spatial board instead of raw JSON text.
- `Esc` exits active node editing or selection mode in a predictable order.
- The status line shows enough context to distinguish canvas view from Fountain/Markdown view.
- Clicking a text node can enter canvas text editing without mutating the raw JSON buffer through the normal text editor path.

M2. Pan, zoom, and fit controls

Status: implemented v1 navigation; explicit fit/reset controls still pending

Implement reference-board navigation.

Rules

- Mouse wheel scrolls vertically or pans according to existing app conventions.
- Ctrl/Cmd plus wheel zooms around the cursor.
- Middle-mouse drag pans the canvas.
- Space plus left drag pans the canvas.
- Opening or reloading a canvas centers the viewport on the content bounds when panel layout information is available.
- `0` or a toolbar command resets zoom to 100 percent.
- Fit-to-content zooms and pans to show all visible nodes.
- Zooming and panning must not move node coordinates.

Acceptance

- The user can comfortably navigate a canvas larger than the window.
- Zoom is centered around the cursor.
- Middle-mouse drag pans without triggering text-pane autoscroll.
- Space plus left drag pans the canvas.
- Opening a canvas starts centered on its content instead of at an arbitrary corner.
- Fit-to-content works for one node, many nodes, and an empty canvas.
- Node positions remain stable across pan/zoom changes.

M3. Selection and interaction modes

Status: partially implemented v1 node hit-testing and movement

Add normal canvas selection behavior before adding advanced tools.

Rules

- Click selects a node.
- Shift-click toggles node selection.
- Dragging empty canvas creates a marquee selection.
- Dragging selected nodes moves them together.
- Ctrl plus left drag on a canvas node moves the clicked node immediately.
- Ctrl plus left drag keeps its existing text-pane scroll behavior outside canvas view.
- Plain left click on a text node enters text editing in v1.
- Plain left click on empty canvas or a non-text node exits active text-node editing.
- Node hit-testing uses reverse node order so the topmost card receives the click.
- Resize handles appear for selected nodes.
- Delete removes selected nodes and selected edges after normal undo/dirty handling is in place.
- Clicking an edge selects the edge.
- Double-clicking a text node may also enter text editing, but v1 supports single-click entry.
- Double-clicking a file node opens the referenced file in the appropriate Basscript view.

Acceptance

- Single selection, multi-selection, marquee selection, drag move, and resize work.
- Ctrl plus left drag moves the clicked card and updates the saved node coordinates.
- Plain click on a text card enters editing instead of panning or moving the card.
- Selection outlines scale correctly with zoom.
- Double-clicking a Markdown or Fountain file node opens the file.
- Double-clicking an image node can either fit the image node or open the image preview, but it must not edit JSON text.

M4. Canvas toolbar

Status: todo

Add a compact toolbar for common canvas actions.

Actions

- Select/move
- Add text node
- Add file/image node
- Add link node
- Add group
- Connect nodes
- Fit to content
- Zoom in/out/reset
- Bring forward/send backward
- Delete

Acceptance

- Toolbar buttons use icons where available and do not use explanatory in-app text.
- Buttons reflect the active tool/mode.
- Disabled actions are visibly disabled when nothing relevant is selected.
- Keyboard shortcuts and toolbar actions call the same command handlers.

Epic N - Canvas Nodes, Images, and Editing

N1. Image and file nodes

Status: partially implemented

Make file nodes useful for visual reference boards.

Rules

- Image file nodes render the actual image with object-fit behavior.
- Canvas image rendering supports `png`, `jpg`, `jpeg`, `webp`, and `bmp` through the Bevy image pipeline.
- The Bevy `jpeg` feature must be enabled so `.jpg` / `.jpeg` canvas images do not log runtime unsupported-format warnings.
- Direct `http://` and `https://` image targets are loaded through the Bevy asset server.
- File node aspect ratio is preserved by default when resizing from corners.
- Holding Shift or using a toolbar toggle can allow free resize.
- File path display is optional and should not cover the image unless selected or hovered.
- Missing or unsupported files show a placeholder with filename and status.
- Image file nodes use the same image cache as Markdown/Fountain embeds.

Acceptance

- Creating a file node for `refs/door.png` displays the image.
- A `.jpg` or `.jpeg` image node renders without `feature "jpeg" is not enabled` warnings.
- A direct remote image URL renders or falls back to a stable placeholder.
- Resizing preserves the image ratio by default.
- Missing image files remain visible as placeholders.
- Repeated image file nodes reuse the same loaded asset.

N2. Text nodes

Status: partially implemented; v1 click-to-edit is implemented

Text nodes provide quick notes on the board.

Rules

- Text node content is stored in the JSON Canvas `text` field.
- Plain left click enters editing in v1.
- `Esc` exits editing.
- Plain typing changes only the active text node.
- `Enter` inserts a newline in the active text node.
- `Backspace` deletes from the end of the active text node in v1.
- Modifier key combinations are ignored by text-node input so global/editor shortcuts do not become literal node text.
- Editing a text node preserves unrelated JSON Canvas fields on the node and document.
- When a text node is in edit mode, image preview rendering is suppressed and the raw node text is shown.
- v1 editing appends at the end of the node text; full caret placement and selection inside a node are pending.
- Basic Markdown styling may reuse existing Markdown processed rendering where practical.
- v1 does not need full rich text or nested editor panels inside nodes.

Acceptance

- Existing text nodes can be edited, moved, saved, and reopened.
- Editing a text node does not affect the main document buffer.
- Editing a text node updates the JSON Canvas `text` field and keeps unknown fields intact.
- `Esc` exits editing and returns the node to normal preview rendering.
- Markdown headings/lists in text nodes render acceptably or fall back cleanly to plain text.

N3. Groups and arrangement

Status: partially implemented

Groups provide spatial organization similar to Obsidian Canvas containers.

Rules

- Group nodes render behind contained nodes.
- Moving a group may optionally move contained nodes in v1, but the behavior must be consistent and documented in status/UX.
- Group labels are editable.
- Group background images use the JSON Canvas `background` and `backgroundStyle` fields when present.
- Bring forward/send backward changes node array order.

Acceptance

- A group loads and renders behind other nodes.
- Group labels save and reload.
- Z-order commands update visual stacking and saved node order.
- Background image fields are preserved even if rendering is deferred.

N4. Edge creation

Status: todo

Let users create simple node connections.

Rules

- A connect tool or node-side handle starts an edge drag.
- Dropping on another node creates an edge.
- Dropping on empty canvas cancels.
- Default new edges use `toEnd: arrow`.
- Edge labels can be added in a later pass, but existing labels must render and save.

Acceptance

- The user can connect two nodes.
- Moving nodes keeps the edge attached.
- Deleting a node removes or hides connected edges according to the chosen data rule, without leaving renderer crashes.

Epic O - Canvas Persistence, Undo, and Integration

O1. Canvas save and dirty state

Status: partially implemented

Canvas files need the same predictable save behavior as text documents.

Rules

- Moving, resizing, creating, deleting, or editing nodes marks the canvas dirty.
- Ctrl plus left drag updates the clicked node's `x` and `y` in the JSON document.
- Canvas text editing updates the active text node's `text` field in the JSON document.
- Canvas coordinate and text updates preserve unknown JSON fields.
- `Ctrl+S`, direct Save, and command-menu `w` save the current canvas path.
- Save As writes a new `.canvas` file.
- Saving rounds coordinates and dimensions to JSON Canvas integer values.
- Save failures show useful status messages.

Acceptance

- Editing a canvas marks it dirty.
- `Ctrl+S` writes a valid `.canvas` without opening Save As when a save path exists.
- Reopening the saved canvas preserves moved node positions.
- Reopening the saved canvas preserves edited text-node content.
- Save failures do not clear dirty state.

O2. Canvas undo/redo

Status: partially implemented v1

Add canvas-specific undo steps after core editing is usable.

Rules

- Node move operations coalesce into one undo step per Ctrl plus left drag.
- Text node edits participate in undo by taking a snapshot on the first text change of an edit session.
- Resize operations participate in undo once resizing is implemented.
- Create/delete node and create/delete edge operations participate in undo.
- Undo/redo does not apply text-buffer edits to canvas files or canvas edits to text documents.

Acceptance

- Dragging a node then undoing returns it to the previous position.
- Editing a text node then undoing restores the previous node text.
- Resizing then undoing restores the previous size once resizing is implemented.
- Creating and deleting nodes can be undone.

O3. Script and Markdown integration

Status: todo

Connect the canvas view back to the writing workflow.

Rules

- A Markdown or Fountain image embed should offer an action to add the referenced image to the current canvas.
- File nodes for `.md` and `.fountain` files open the referenced text document on double-click.
- Explorer drag/drop into the canvas creates file nodes.
- Dragging images from the explorer into the canvas creates image file nodes.
- Dragging Markdown/Fountain files from the explorer into the canvas creates file preview nodes.
- Canvas search should include text node contents, file node paths, and link URLs.

Acceptance

- An image referenced in a Fountain script can be placed on a canvas without retyping the path.
- Dragging an image from explorer to canvas creates an image node.
- Dragging a script file from explorer to canvas creates a file node that opens the script on double-click.

Epic P - Verification and Test Coverage

P1. Parser and resolver tests

Status: partially implemented

Acceptance

- Core parser tests cover valid image syntax, invalid image syntax, Markdown code fences, Fountain image-only lines, and script-link coexistence.
- Path resolver tests cover relative document paths, workspace-relative fallback, absolute paths, missing files, and unsupported extensions.
- Canvas model tests cover valid JSON Canvas load/save, unknown field preservation where implemented, duplicate IDs, dangling edges, and invalid JSON.
- Canvas model tests cover blank canvas files, UTF-8 BOM stripping, node-position updates that preserve fields, and text-node updates that preserve fields.

P2. Rendering smoke tests

Status: todo

Acceptance

- Processed Markdown image embed smoke test proves image entities are spawned and positioned.
- Processed Fountain image embed smoke test proves image blocks reserve vertical space.
- Canvas smoke test proves `.canvas` files render nodes at expected coordinates.
- Pan/zoom smoke test proves node screen positions change with viewport transform while document coordinates remain unchanged.

P3. Manual QA checklist

Status: todo

Checklist

- Open Markdown file with one local image.
- Open Fountain file with one local image.
- Switch Raw, Processed, Split, and Focus modes.
- Edit image path and verify placeholder/image updates.
- Open `.canvas` file with image, text, link, group, and edge nodes.
- Open a `.canvas` file that starts with a UTF-8 BOM and verify it parses.
- Open a blank `.canvas` file and verify it shows an empty board.
- Pan with Space plus left drag and middle-mouse drag.
- Move a canvas card with Ctrl plus left drag, save, close, and reopen.
- Click a text card, edit it, press `Esc`, save, close, and reopen.
- Put a Markdown image embed in a canvas text card and verify it renders as an image preview outside edit mode.
- Put an HTML `<img src=...>` tag in a canvas text card and verify it renders as an image preview outside edit mode.
- Open a canvas with a `.jpg` or `.jpeg` image node and verify no JPEG feature warning is logged.
- Pan, zoom, fit content, select, move, resize, save, close, reopen.
- Delete or rename a referenced image and verify placeholders.

Implementation notes

Likely implementation areas:

- `core/src/model.rs` for `ImageEmbed`, `ParsedLine.image_embeds`, and possibly a Canvas document model or new `DocumentFormat::Canvas`.
- `core/src/parser/shared.rs` for shared image embed extraction and script-link exclusion.
- `core/src/parser/markdown.rs` for fenced-code suppression and Markdown image classification.
- `core/src/parser/fountain.rs` for image-only line classification and dialogue-context reset.
- `ui/src/editor/processed.rs` for processed visual blocks that can reserve more than one text line and contain image entities.
- `ui/src/editor/rendering/shared.rs` for spawning/updating processed image nodes, placeholders, hit-testing, selection, and caret mapping around image blocks.
- `ui/src/editor/rendering/markdown.rs` and `ui/src/editor/rendering/fountain.rs` for format-specific visual text/image decisions.
- `ui/src/editor/ui_setup.rs` for reusing or extending existing image/SVG asset loading helpers.
- `ui/src/editor/core.rs` for document-format routing, image cache resources, canvas view state, dirty state, commands, and status messages.
- `ui/src/pannels/text/explorer.rs` and `ui/src/pannels/text/explorer_actions.rs` for opening `.canvas` files and drag/drop into documents or canvases.
- `core/src/canvas.rs` for JSON Canvas model parsing, BOM/blank handling, and JSON-preserving field updates.
- `ui/src/editor/canvas.rs` for canvas document state, viewport helpers, node movement, panning, and text-node editing input.
- `ui/src/editor/rendering/canvas.rs` for canvas board rendering, node/edge drawing, card image previews, and canvas-specific image target extraction.
- `ui/src/editor/rendering/shared.rs` for the shared processed/canvas image lookup, local image cache, and remote image asset loading.
- `settings/state.ron` for persisted canvas viewport preferences if needed.

Suggested implementation order

1. K1 shared image parser
2. K2 image path resolver
3. K5 image asset cache
4. K3 processed Markdown image layout
5. K4 processed Fountain image layout
6. K6 image insertion workflow
7. L1 and L2 canvas document/file-node model
8. M1 and M2 canvas view shell, pan, zoom, fit
9. N1 image/file nodes
10. M3 selection, move, resize
11. O1 save/dirty state
12. N2 text nodes
13. L3 and N4 edges
14. N3 groups and z-order
15. O2 undo/redo
16. O3 script/Markdown integration
