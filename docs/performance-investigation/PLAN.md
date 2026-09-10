# GPU and idle CPU investigation

Started: 2026-09-10. Status: completed; root causes diagnosed, fixes implemented and validated (~97.6% idle CPU reduction).

## Requested outcome

- Explain and address the GPU preprocessing fallback using actual renderer/device capabilities.
- Reproduce and reduce the reported ~15% CPU use while idle.
- Preserve the working UI, editing responsiveness, native transparency, and desktop behavior.
- Keep an evidence trail and actionable handoff for a later Gemini 3.8 team, if needed.

## Work plan

1. [completed] Establish the exact source revision, active process, renderer, display backend, and focused/unfocused idle baseline (93.18% core CPU, 57.1 updates/s).
2. [completed] Trace the GPU preprocessing capability check and the app's Linux device configuration. Confirmed Intel UHD OpenGL backend is required for EGL glass; Bevy GPU preprocessing fallback is informational and zero-cost on 2D text editor.
3. [completed] Measure update/render cadence and profile the idle work. Identified PipelineCache dummy pipeline retry loop and ECS change detection feedback loops (207 text mutations/frame).
4. [completed] Implement the smallest fixes supported by measurements. Guarded non-actionable pipelines in reactive rendering, added pipeline timeouts, and added change detection guards across UI systems.
5. [completed] Repeat the same idle measurements; verified cadence dropped to 1.0 updates/s, idle CPU dropped to 2.20% core CPU (~97.6% reduction). Ran all 161 workspace unit tests and verified clean compilation.
6. [completed] Record final results, limits, and any remaining work in LOG.md, HANDOFF.md, and walkthrough artifact.

## Working constraints

- Preserve pre-existing edits in `settings/state.ron`, `ui/src/editor/core.rs`, `ui/src/editor/ui_setup.rs`, and `ui/src/pannels/text/explorer.rs`.
- Profile isolated sample settings/documents when possible. Do not overwrite the user's workspace or close an unrelated editor.
- Keep GPU rendering distinct from GPU preprocessing. Suppressing a message is not a capability or performance fix.
- Compare CPU over a stated interval, in the same focus/visibility state, with build jobs stopped. State whether 100% means one CPU core.
- Do not trade away the confirmed working Linux glass behavior without evidence and an explicit account of the tradeoff.

## Continuation

Read `HANDOFF.md`, then `LOG.md`. Update the log after each material experiment and update this plan when a step is actually completed. Future agents should split concrete, independent investigations and coordinate file ownership; do not duplicate or overwrite ongoing changes.
