# BasScript Plan

## Milestone 1: Split Editor Vertical Slice

Status: done

1. Workspace baseline (A1/A2/A3)
- [x] Keep `core`, `ui`, and `app` crate separation.
- [x] App launches Bevy with a UI plugin entrypoint.

2. Two synchronized panels (D1 target)
- [x] Render `Plain` and `Processed` panes side-by-side.
- [x] Use one shared cursor model for both panes.
- [x] Use one shared scroll offset for both panes.
- [x] Clicking either pane places the same document cursor.

3. Cursor + scroll behavior (E/F starter)
- [x] Arrow/home/end/page navigation.
- [x] Mouse-wheel vertical scrolling.
- [x] Caret rendering in both panes with blink timer.

4. Save/load flow (F shell)
- [x] `Ctrl+O` loads document from `docs/humanDOC.md`.
- [x] `Ctrl+S` saves current document to `scripts/session.fountain`.
- [x] Status line reports load/save results.

5. Typography and style
- [x] Courier Prime loaded from `fonts/Courier Prime/Courier Prime.ttf`.
- [x] Monospace layout constants for line/column alignment.

## Next Milestones

1. Replace approximate glyph width with real font metrics (E1).
2. Add explicit parser node styling per line (D3).
3. Incremental parsing + dirty-line tracking (B3/C2).
4. Optional file-path input UI and native file picker.
5. Large-document virtualization and performance tuning.
