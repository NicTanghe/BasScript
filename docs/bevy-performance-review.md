# Bevy and performance review

Reviewed on 2026-09-07 against the locked Bevy 0.19.0 sources and the
[official 0.19 migration guide](https://bevy.org/learn/migration-guides/0-18-to-0-19/).
The review focused on dependency selection, document editing, text rendering,
ECS change detection, scheduling, and workspace/save paths. This is not a claim
that every code path is optimal or that desktop interactions were profiled.

## Implemented

- **Use Bevy's UI feature collection.** Both crates inherit a single workspace
  dependency with `default-features = false` and `ui`. Supported image formats,
  HTTP(S) assets, desktop window backends, and optional dynamic linking remain
  enabled. The application no longer enables the unused 3D and audio stacks;
  26 packages were removed from the lockfile. The ability to select `ui` without
  audio is part of Bevy 0.19's feature collection changes.
- **Insert pasted text in bulk.** `Document::insert_text` splits at newlines,
  scans UTF-8 positions once, and splices new lines together. It preserves the
  original character-indexed cursor, suffix, and newline behavior. The old loop
  repeatedly scanned and shifted text for every pasted character.
- **Share processed layout snapshots.** Cached visual lines use `Arc<[...]>`.
  Rendering, selection, scrolling, and image systems can obtain the same layout
  without deep-copying the document. Cache rebuilds preserve existing snapshots.
  `build_processed_view` copies only its requested window and EOF padding.
  Individual visual fragments are borrowed unless overflow fragments must merge.
- **Preserve Bevy's change detection.** Font helpers keep the `Mut<TextFont>`
  wrapper rather than coercing it to `&mut TextFont`, which marked every span
  changed before the helper could compare values. Document and query text use
  `map_unchanged`, `set_if_neq`, and `clone_from_if_neq`. Canvas text metrics and
  padding are also compared before assignment. Selected visibility and UI
  updates avoid redundant writes. A shared query alias reduces duplication.
- **Keep an empty document editable.** `Document::default()` now creates the
  same initial empty line as `Document::new()`; the previous derived default
  produced a zero-line buffer that panicked when edited.
- **Keep the editor open after a failed `:wq` save.** Save methods report
  success, and write-and-quit emits `AppExit` only after saving succeeds.
  A failed save keeps the error message and document available.
- **Enable the existing clipboard adapter on macOS.** Its previous platform
  branches always returned `None`/`false` on macOS. The shared Windows/macOS
  dependency now includes `arboard`; Linux retains its Wayland feature.
- Simplify the alias-resolution branch and conditional image parsing without
  changing their behavior.

## Measured buffer performance

Median of five runs on this Linux machine with Rust 1.97 and `rustc -O`.
Each case inserts at line 0, column 4 into a document containing
`"original text\n".repeat(10_000)`. Cloning the initial document and constructing
the pasted string happen outside the measured region.

| Pasted input | Before | After |
| --- | ---: | ---: |
| `"é🙂a".repeat(10_000)` — 70,000 bytes | 1,316.19 ms | 0.016 ms |
| `"A new screenplay line.\n".repeat(10_000)` — 230,000 bytes | 125.53 ms | 0.635 ms |

These are buffer-operation measurements, not end-to-end input latency or frame
rate. Parsing, history snapshots, and rendering still contribute to UI latency.

## Follow-up work, in priority order

1. **Move workspace scanning/indexing off the UI update path.**
   `refresh_workspace_after_path_change` triggers a full workspace refresh on
   saves within the workspace. `scan_workspace_files` builds entity, scene, and
   appearance indexes synchronously, even when few files changed. Use incremental
   indexing and Bevy task pools with explicit completion handling; preserve the
   current database transaction and error-reporting behavior.
2. **Reduce idle work and allocate pages to fit the viewport.** The processed
   editor and query sheet each preallocate 16 pages with 24 spans per row.
   `render_editor` still runs each frame, and several UI nodes and the large
   `EditorState` resource are still mutated every frame. Profile long scripts
   and large Canvases, split independently changing state into resources, and
   make layout/presentation updates respond to those changes. The
   `ProcessedRawCurrentLine` path still rebuilds its layout, and link-target
   discovery still scans parsed lines on repeated layout requests.
3. **Introduce explicit input/model/presentation system sets.** Many systems
   share mutable `EditorState`; registration order does not establish execution
   order. Preserve the existing focus/capture constraints while making the
   intended ordering clear and testing input-to-render behavior.
4. **Prototype `EditableText` in a simple field.** Start with the command menu
   or workspace prompt and test Enter/Escape, focus capture, Unicode, selection,
   and clipboard behavior. Bevy 0.19 supports these building blocks, but its
   native text control does not yet provide undo/redo. Keep the rich document
   model, Vim behavior, and bundled font metrics. See the existing
   [0.19 port notes](specsheet_0.19_port.md) and
   [official text-input notes](https://bevy.org/news/bevy-0-19/#text-input).
5. **Add an idle redraw policy after accounting for timers.**
   `WinitSettings::desktop_app()` alone would delay the app's caret blinking,
   animated controls, held-key navigation, and polled dialog results. A reactive
   policy needs redraw requests or wakeups for these cases and asset loading.
   Startup, asset/pipeline settling, and caret timing now have a
   [reactive rendering policy](reactive-rendering.md); retain the documented
   idle intervals until the remaining timer/input paths have completion wakeups.

BSN and Feathers may simplify future UI construction. Replacing the existing
`children!` bundles by itself would not establish a runtime performance gain.
Settings migration and system font substitution also need their own compatibility
checks; the existing formats and PDF/layout metrics should guide those choices.

## Validation and limits

The existing 176 tests passed before editing. The updated regression suite has
183 tests covering the document, parsing, links, index, PDF export, typography,
Canvas geometry, caching, and failed-save behavior. The new ECS regression checks
that a second unchanged style update does not mark text, font, line-height, or
color components changed, then confirms that a real edit updates the text/font.

The Linux application build, formatting check, and diff whitespace check also
pass. The checks can be reproduced with:

```sh
cargo build --package basscript-app --locked
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked
git diff --check
```

Clippy completes, including the optional dynamic-linking feature, but the
repository is not warning-free: existing query-type complexity, large function
signatures, and style warnings remain. A strict `-D warnings` check already
failed on the baseline. These have not been hidden with broad lint suppressions.

Validation is on Linux. Windows/macOS builds and interactive GPU, resize, IME,
clipboard, and drag/selection smoke tests remain necessary before a release.
In particular, the macOS clipboard change has not been exercised on macOS.
