# Reactive rendering

`app/src/reactive_rendering.rs` keeps both focused and unfocused windows in
Bevy's `UpdateMode::Reactive`. It does not switch to continuous rendering or
change the graphics backend, presentation mode, or pipelined renderer.

The event loop starts with a 16 ms wait, including before the first `Startup`
systems run. A redraw request from `Update` cannot drive initialization before
that schedule is running. The previous fixed burst of 30 redraw requests also
could not account for delayed font loading or pipeline compilation.

Short reactive waits continue while rendering pipelines are pending and for
250 ms after font, image, shader, window-size, or display-scale changes. The
render world shares pipeline readiness with the main world through an atomic
flag. Settling uses wall-clock time, including time since the last render
timestamp, so returning from idle does not prematurely expire the settling
period.

Once settled, the idle waits are 500 ms when focused and 1 second when
unfocused. Input and explicit redraw requests still wake the application.
Asset results that arrive while idle are picked up on the next update, then
receive the short settling cadence. Caret blinking uses `Time<Real>` and
preserves its phase when one update spans multiple blink intervals.

These idle waits also bound latency for the editor's existing polled background
work. Do not increase them to seconds/minutes without adding completion wakeups
and checking held-key navigation and UI animations.

## Verification

Run `cargo test --workspace --locked --offline`. Regression tests cover:

- Reactive settings in both focus states, startup settling, and stable idle.
- Pipeline compilation lasting longer than 30 frames.
- Late asset, resize, and DPI events, including simultaneous event streams.
- Settling after an idle render timestamp and real-time caret blinking.

A Linux/X11 smoke test on NVIDIA RTX 2070/Vulkan displayed the editor content
and returned to idle after startup and subsequent activity. Debugger inspection
confirmed the main thread waiting in the native event loop while idle.
Windows and macOS have not been smoke-tested for this change.

For transition logs, run with
`RUST_LOG=info,basscript_app::reactive_rendering=debug`.
