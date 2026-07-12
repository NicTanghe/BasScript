# BasScript WYSIWYG PDF Export Specsheet

Status: implemented and verified

## Goal

Export the current document as a searchable, font-embedded PDF whose page breaks,
line wrapping, indentation, typography, colors, checklist state, and local images
match BasScript's paginated processed view as closely as PDF viewers permit.

The fidelity contract is geometric rather than screenshot-based:

- one unzoomed processed-layout unit equals one PDF point;
- A4 is exactly 210 x 297 mm (`595.2756 x 841.8898 pt`);
- editor zoom, operating-system scale, monitor DPI, window size, scroll position,
  caret, selection, formatting marks, and hover state never affect export;
- source text is emitted as PDF text with the same bundled font files used by the
  editor, rather than rasterizing whole pages;
- export always uses paginated processed layout, even when continuous mode is
  selected on screen.

This is WYSIWYG at the document-layout layer. Pixel-identical antialiasing is not
possible across Bevy, PDF renderers, monitors, and printers, but positions and
sizes must agree in point space.

## Non-goals

- exporting the plain source pane, editor chrome, caret, selection, or glass;
- using the monitor's physical DPI to choose print dimensions;
- reproducing a PDF viewer's glyph rasterization pixel-for-pixel;
- silently downloading remote images during export;
- exporting Canvas documents in the first implementation. Canvas has a different,
  spatial page model and must report a clear unsupported-format error.

## Epoch 1 - Canonical geometry and export snapshot

Status: complete

Create an immutable export snapshot from the current `EditorState`.

The snapshot owns:

- exact A4 width and height in points;
- current persisted page margins in points;
- document format and source path;
- processed `wrap_columns`, `lines_per_page`, and visual lines;
- an explicit page/line structure independent of viewport virtualization;
- the resolved style and fragments for every visible line.

Requirements:

- build from the same processed-line preparation, wrapping, indentation, inline
  emphasis, double-space handling, image reservation, and page-fill logic used by
  the screen;
- never use the current raw-line editing override;
- use exact A4 height for the PDF MediaBox while retaining the existing line-grid
  pagination contract;
- omit spacer rows and front matter exactly as processed view does;
- produce at least one page for an empty document;
- reject Canvas with an actionable message.

Acceptance:

- unit tests prove A4 point dimensions and point-to-mm conversion;
- snapshot page count and line assignment match processed pagination fixtures;
- output is invariant under editor zoom and scroll position.

## Epoch 2 - Searchable text and embedded typography

Status: complete

Generate PDF pages directly from the snapshot.

Requirements:

- embed the bundled Courier Prime regular/bold/italic/bold-italic files for
  Fountain documents;
- embed the bundled Segoe UI files used by Markdown for the four variants;
- map the processed base style plus inline emphasis to the same font variant;
- apply `FONT_SIZE * font_scale` in PDF points;
- use the same line-top progression and `LINE_HEIGHT * line_height_scale` rules;
- convert the screen's top-left coordinates to PDF's bottom-left coordinates;
- preserve processed colors, including configured link colors when enabled;
- emit Unicode text through embedded fonts and retain searchable/copyable text;
- subset fonts when supported by the PDF backend.

Text placement contract:

- each processed line begins at `page_margin_left`;
- its vertical cell begins at `page_margin_top + accumulated_line_height`;
- PDF baseline placement is derived from the bundled font metrics, not a guessed
  monitor DPI conversion;
- fragment advances are derived from the same font data used for export. For the
  monospaced Fountain layout, the established `7.2 pt` cell contract remains the
  wrapping/indentation authority.

Acceptance:

- all four variants are represented in a fixture PDF;
- headings retain their processed font and line-height scaling;
- extracted PDF text contains the expected document text;
- page size is exact A4 and font size is 12 pt at base scale.

## Epoch 3 - Links, checklist marks, and images

Status: complete

Requirements:

- preserve inline link text styling;
- add URI annotations for `http://` and `https://` targets;
- do not expose internal BasScript link targets as broken external URLs;
- draw checklist boxes/check marks as PDF vector geometry at the same line anchor
  used by processed view;
- embed local PNG, JPEG, WebP, and BMP images supported by the application;
- resolve relative image paths with the same document/workspace rules as the
  processed view;
- fit images inside the processed image reservation while preserving aspect ratio;
- center fitted images in the reservation and never upscale beyond its box unless
  the on-screen contract does so;
- replace missing, unsupported, or remote images with a small visible placeholder
  and accumulate a non-fatal warning.

Acceptance:

- image aspect ratio and reservation bounds are tested without inspecting pixels;
- missing images do not abort an otherwise valid export;
- a PDF with images remains readable and reports warnings in the status line;
- external URL annotations cover the corresponding visible text.

## Epoch 4 - Application integration

Status: complete

Requirements:

- add an `Export PDF` toolbar action;
- open a native save dialog filtered to `.pdf`;
- suggest the current document stem with a `.pdf` extension;
- append `.pdf` when the chosen path has no extension;
- never replace the current source document path with the PDF path;
- run PDF construction and file writing after dialog completion;
- use an atomic same-directory temporary file followed by rename where supported;
- report success with page count and warnings; report failures without panicking;
- reject a second dialog while another native dialog is pending.

Acceptance:

- cancel leaves state and files unchanged;
- success status includes the destination and page count;
- write/rename failures are surfaced clearly;
- Save and Save As behavior remains unchanged.

## Epoch 5 - Verification and regression protection

Status: complete

Automated checks:

- geometry, pagination, font mapping, image fitting, extension normalization, and
  snapshot invariance tests;
- PDF smoke test checks `%PDF-`, page count, exact MediaBox dimensions, embedded
  font resources, and representative text;
- full workspace formatting, tests, and compile checks.

Manual visual comparison protocol:

1. Open the fixture at 100% zoom in paginated processed view.
2. Export it, render the PDF page at 72 DPI, and compare page-space coordinates.
3. Verify page edges, margin anchors, baselines, wrapping, page breaks, variants,
   colors, checklist marks, and image boxes.
4. Repeat with Fountain and Markdown, non-default margins, and 60/180% editor zoom.
5. The exported geometry must not change with editor zoom.

Tolerance:

- page and explicit layout coordinates: <= 0.01 pt;
- text baseline/advance differences caused by independent shaping engines:
  <= 0.5 pt per run and no changed wrap or page break;
- fitted image bounds: <= 0.1 pt;
- raster appearance is assessed visually and is not a byte/pixel equality promise.

## Known boundaries

- PDF viewers may rasterize, hint, and antialias the same embedded font differently
  from Bevy.
- The existing processed view wraps by established format-specific character-cell
  widths. Export preserves those breaks rather than reflowing with PDF measurements.
- Exact A4 height and the editor's 12-point line grid do not divide evenly. The PDF
  keeps exact A4 paper size and the processed view's current line-grid pagination,
  leaving the sub-line remainder at the bottom of the page.
- Remote images are intentionally not fetched as part of export.
