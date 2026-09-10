# Continuation handoff

## Current state
 
The investigation into high idle CPU and the GPU preprocessing message is **completed**. Root causes were diagnosed, fixes were implemented and verified with live measurements:
- **Idle CPU**: dropped from **93.18%** core CPU to **2.20%** core CPU (~**97.6% reduction**).
- **Idle Cadence**: dropped from **57.1 updates/s** to **1.0 updates/s** (`Reactive { wait: 1s }`).
- **All 161 workspace unit tests pass** offline (`cargo test --workspace --locked --offline`).
- **Linux glass transparency and blur remain fully intact**.

## Summary of Findings & Implemented Fixes

1. **GPU Preprocessing Fallback**:
   - In Bevy 0.19, `bevy_render::batching::gpu_preprocessing` hardcodes disabling GPU preprocessing when `adapter_info.backend == Backend::Gl`.
   - This message is purely informational. GPU preprocessing in Bevy is a 3D mesh culling compute pass. In a 2D text editor, it takes microseconds on the CPU and has zero performance cost.
   - Mesa Intel UHD is the display adapter connected to the X11 screen and is required for native EGL 32-bit ARGB glass presentation. Forcing NVIDIA offload on X11 causes an unrecoverable EGL present mode panic in `wgpu-hal`.
2. **PipelineCache Infinite Retry Feedback Loop**:
   - `SparseBufferPlugin` in Bevy 0.19 specializes a dummy compute pipeline (`Handle::default()`) when storage buffers < 3 on OpenGL. `PipelineCache` retries it endlessly every frame with `Err(ShaderNotLoaded)`.
   - `track_pending_pipelines` in `app/src/reactive_rendering.rs` was fixed to filter out non-actionable pipelines (`"sparse buffer update pipeline"` and `Handle::default()`) and enforce a pipeline timeout.
3. **ECS Change Detection Churn**:
   - Multiple UI systems (`sync_hovered_processed_link`, `sync_settings_ui`, `sync_theme_picker_ui`, `blink_caret`, `render_selection_rects`, etc.) were mutating ECS components unconditionally every frame, defeating Bevy's change detection and generating hundreds of `AssetEvent` updates.
   - Systems were updated with state change guards, focus checks, and `set_if_neq` assignments.

## Relevant files

- `app/src/linux_renderer.rs`: manual Linux EGL device initialization and requested limits.
- `app/src/reactive_rendering.rs`: idle/settling cadence and asset/pipeline wakeups.
- `app/src/main.rs`: native window, renderer selection, plugin setup.
- `ui/src/editor/core.rs`: editor systems and schedules.
- `ui/src/editor/rendering/shared.rs`, `metadata_controls.rs`, `ui_setup.rs`, `caret.rs`: likely sources of repeated UI work and redraws.
- `docs/window-glass.md`, `docs/reactive-rendering.md`, `docs/bevy-performance-review.md`: prior decisions and known limitations.

## Useful commands

```sh
cargo test --workspace --locked --offline
cargo build -p basscript-app --locked --offline
```

Use `RUST_LOG=info,basscript_app::reactive_rendering=debug` for settling/idle transitions. GUI launches and host process profiling can require sandbox escalation; use the tool's permission mechanism. Existing GUI preview assets/settings are in `/tmp/basscript-ui-polish`, but inspect their current contents and process state before reuse.

## Potential future parallel work

- Renderer capability investigation and tests (own `app/src/linux_renderer.rs`).
- Event-loop wakeup/cadence profiling (own `app/src/reactive_rendering.rs`).
- Repeated UI work/change-detection profiling (coordinate ownership of individual UI files).
- Integrator handles shared documentation, builds, and before/after native measurements.

These are suggested continuation tasks, not a claim that agents have been started or that Gemini is available in this session.
