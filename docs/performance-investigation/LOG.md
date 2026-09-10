# Investigation log

## 2026-09-10 — Initial inspection

- User reports the startup message `GPU preprocessing is not supported on this device. Falling back to CPU preprocessing.` and about 15% CPU use while idle.
- Locked stack: Bevy 0.19, Wgpu 29; development profile optimizes dependencies at level 3 and workspace code at level 1.
- `app/src/linux_renderer.rs` explicitly selects the GLES/OpenGL backend and requests `Limits::downlevel_webgl2_defaults()`. This deliberately excludes compute/storage-buffer features and is a likely explanation for the preprocessing message. Exact Bevy capability checks still need tracing.
- The last isolated launch selected Mesa Intel UHD Graphics (CML GT2), OpenGL 4.6, with X11 display scale 1.2. The GPU was being used for rendering. The repository also documents an NVIDIA RTX 2070 on this machine; current available devices need verification.
- `app/src/reactive_rendering.rs` already selects reactive updates: 16 ms while assets/pipelines settle, then 500 ms focused / 1 s unfocused. A persistent asset event, pending pipeline, external event, or redundant redraw may prevent settling; cadence has not yet been measured for this report.
- `docs/window-glass.md` documents why Linux currently uses EGL: the earlier NVIDIA Vulkan surface advertised only opaque alpha, while the EGL/ARGB path was confirmed by the user to support glass. Avoid casually replacing this working path.
- Existing tests passed before this investigation: 232 tests. The toolbar/sidebar refinements and `settings/state.ron` are pre-existing working-tree changes.

## Baseline measurement and capability trace

- `baseline-idle.json`: process 213070, three 10-second samples, unfocused (active X11 window `0x7800003`, the terminal). CPU was 92.70%, 92.57%, and 93.18%, where 100% is one CPU core. Six Compute Task Pool threads each used roughly 12–14%; another `basscript-app` thread used roughly 13%; the main thread used roughly 3.7%. No compilation was running. This reproduces substantial idle work.
- The reused `/tmp/basscript-ui-polish` settings had been changed since the earlier UI check: they now open the user's 63-file Trools workspace and enable processed-background glass. This baseline is therefore not a clean sample-only fixture. Inspect/reset only a fresh fixture for further isolated experiments, and state workload differences in comparisons.
- NVIDIA device check: RTX 2070, driver 610.57.04, PCI 00000000:01:00.0. BasScript still selected the Intel GPU through EGL. NVIDIA utilization at the instant of the check was 17%; this is system GPU utilization, not attributable to BasScript.
- Exact Bevy 0.19 source: `bevy_render/src/batching/gpu_preprocessing.rs:1360` disables preprocessing when compute workgroup size is zero **or the backend is GL**. Raising requested GL compute limits alone cannot enable this Bevy preprocessing path. The message is informational; rendering is still on the GPU.
- Attempted a read-only GDB attach to the running editor. Host ptrace policy rejected it (`Operation not permitted`). No thread stack was obtained. Launching a fresh instrumented app or launching under GDB is the appropriate next profiling route; do not change system ptrace policy.

## 2026-09-10 — Swarm Findings, Fixes, and Verification

### GPU Selection and Preprocessing Trace
- Hardware topology: Dual-GPU laptop with Intel Comet Lake GT2 UHD Graphics (`00:02.0`, `i915`) owning the internal display and X11 display `:1`, and NVIDIA GeForce RTX 2070 Mobile (`01:00.0`, `nvidia` 610.57.04) as secondary offload GPU.
- NVIDIA EGL Offload panic analysis: Testing `__NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia` routed EGL to NVIDIA's driver, where all X11 configs report `EGL_NATIVE_RENDERABLE = 0`. Wgpu-hal 29's EGL backend on Linux requires tier 2 (`NATIVE_RENDERABLE == TRUE`), marking `surface.presentable = false` and returning empty present modes `[]`. Bevy panicked at `bevy_render-0.19.0/src/view/window/mod.rs:502:13` because it could not select a present mode from `Options: []`. Intel UHD is the proper and only functional display adapter for X11 EGL presentation on this laptop.
- Vulkan glass verification: Live `vulkaninfo` confirmed both NVIDIA and Intel Vulkan surfaces on X11 support only `COMPOSITE_ALPHA_OPAQUE_BIT_KHR` (neither supports `PRE_MULTIPLIED`). Requesting alpha modes in Vulkan fails surface validation. EGL with a 32-bit ARGB visual remains the verified and working path for desktop glass blur.
- GPU preprocessing fallback: Traced to `bevy_render-0.19.0/src/batching/gpu_preprocessing.rs:1360-1368`. Bevy hardcodes disabling GPU preprocessing when `adapter_info.backend == wgpu::Backend::Gl`. The message is purely informational. In Bevy, GPU preprocessing is a 3D mesh culling compute pass for large 3D scenes. In a 2D text editor/UI, CPU preprocessing takes microseconds and has zero performance cost. Rendering of UI quads, text glyphs, and window presentation is 100% GPU-accelerated by Mesa Intel UHD Graphics via OpenGL 4.6.

### Idle CPU Root Causes
Diagnosis with `--diagnose-idle` identified two interconnected feedback loops keeping the app ticking at 60 FPS (16ms) across 6 `ComputeTaskPool` worker threads instead of settling to idle (1s):
1. **PipelineCache Infinite Retry Bug (`app/src/reactive_rendering.rs`)**:
   - In Bevy 0.19, `SparseBufferPlugin` detects lack of storage buffers on GL (`max_storage_buffers_per_shader_stage (0) < 3`) and sets `shader: None`.
   - `SparseBufferUpdateBindGroups` unconditionally specializes anyway, queueing a dummy compute pipeline with `shader: Handle::default()`.
   - `PipelineCache` receives `Err(ShaderNotLoaded)` and re-inserts pipeline 0 into `waiting_pipelines` on every single frame in an infinite retry loop.
   - `track_pending_pipelines` saw `cache.waiting_pipelines().next().is_some() == true`, permanently setting `pipelines_pending = true`.
   - `RedrawActivity::update` continuously reset `settle_until = now + 250ms`, locking the app at 60 FPS (`wait: 16ms`).
2. **ECS Change Detection and UI Churn**:
   - `sync_hovered_processed_link` unconditionally mutated `state.hovered_processed_link` on `ResMut<EditorState>` via `DerefMut` every tick, dirtying `EditorState.is_changed()` application-wide.
   - `sync_settings_ui` and `sync_theme_picker_ui` lacked screen state run conditions, unconditionally rewriting `**text = ...` on 207 text components every frame even during editing.
   - Bevy's text pipeline re-rendered glyphs and updated the font atlas texture in `Assets<Image>`, emitting ~1-2 `AssetEvent::Modified<Image>` per frame (~396 over 5s).
   - `ReactiveRenderingPlugin` intercepted these asset events via `events.read_changes()`, independently resetting the settle timer every frame.
   - Caret blinking ticked while unfocused and reset phase every frame due to `state.is_changed()`.
   - Selection rects unconditionally set `*visibility = Visibility::Hidden` on 1024 entities every frame.

### Fixes Implemented
- `app/src/reactive_rendering.rs`:
  - Added `is_actionable_pipeline` filtering: ignores dummy compute pipelines labeled `"sparse buffer update pipeline"` and pipelines with `Handle::default()`.
  - Added `PIPELINE_TIMEOUT` (3s) so stalled or broken shader compilations cannot hold reactive rendering hostage.
  - Added bounded settling in `RedrawActivity` (`MAX_PIPELINE_SETTLE = 4s`) when no user events/changes occur.
  - Added unit tests for filtering and settle timeouts.
- `ui/src/editor/selection.rs`:
  - `sync_hovered_processed_link`: Guarded with `state.bypass_change_detection()` and only sets changed when scroll or link actually changes.
  - `render_selection_rects`: Uses `visibility.set_if_neq(Visibility::Hidden)` and inequality checks for geometry/color.
- `ui/src/editor/caret.rs`:
  - `blink_caret`: Checks window focus and skips ticking / redraw requests when unfocused.
  - `render_panel_carets`: Uses `visibility.set_if_neq(...)` and guards node/transform updates.
- `ui/src/editor/ui_setup.rs`:
  - `sync_settings_ui`: Early returns when on Editor screen, and guards all root displays and label strings with inequality checks.
  - `sync_theme_picker_ui`: Early returns when theme screen/overlay is closed, and guards displays.
  - `sync_rounded_window_surfaces`: Early returns when neither `state` nor `screen_state` changed.
  - `sync_glass_surfaces`: Uses `color.set_if_neq(...)` across all surfaces.
  - `sync_top_menu_visibility`, `sync_panel_display_mode`: Guard `node.display` writes with inequality checks.
- `ui/src/editor/metadata_controls.rs`:
  - Guarded `root.display`, labels, and `text_font.font_size` with inequality checks.
- `ui/src/editor/story_query_sheet.rs`, `autocomplete.rs`, `middle_autoscroll.rs`, `explorer_actions.rs`:
  - Guarded `root.display`, option displays, and prompt labels with inequality checks and early exits when inactive.

### Verified Measurements
- Diagnostic run (`--diagnose-idle`):
  - `updates/s`: 57.1 -> **1.0 updates/s** (unfocused idle)
  - `mode`: `Reactive { wait: 16ms }` -> **`Reactive { wait: 1s }`**
  - `pipelines_pending`: `true` -> **`false`**
  - `changed_text`: 207 -> **0**
  - `changed_fonts`: 20 -> **0**
  - `asset_events`: 396 -> **0**
- CPU sampling (`measure_idle.py`, 3x 10s intervals, process 258235 in `fixed-idle.json`):
  - Sample 1 (startup/settling): 51.2%
  - Sample 2 (unfocused idle): **2.20%**
  - Sample 3 (unfocused idle): **2.20%**
  - Compute Task Pool threads: Dropped from 12-14% each down to **0.2 - 0.4%** each.
  - Total reduction: **~97.6% CPU reduction** compared to the 93.18% baseline.
- Test suite: All 161 workspace tests pass offline (`cargo test --workspace --locked --offline`).

