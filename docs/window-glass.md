# Desktop window glass

BasScript's existing glass switches now work with Linux window transparency.
The desktop compositor supplies background blur; BasScript supplies transparent
pixels and, on Linux, a translucent theme tint. Linux uses an OpenGL/EGL renderer
initialized with the native display connection. This does not require a Bevy fork.

## Enable it

Open **Settings → Theme** and enable any of:

- **Processed background glass**: processed text background and paper.
- **Explorer glass**: the workspace sidebar.
- **Settings glass**: settings, keybindings, theme screens and the top menu.

The switches are also saved in `settings/theme.ron`:

```ron
processed_glass: true,
explorer_glass: true,
settings_glass: true,
```

Edit these fields inside the existing theme object while BasScript is closed,
then launch `cargo run --locked --offline -p basscript-app`. The in-app switches
take effect immediately. All three remain off by default.

Linux glass uses each surface's theme color, with alpha capped at `0.72`.
Lower the color's alpha in Theme for more transparency. Text, controls, the
plain editor, the status line and the Canvas board retain their normal styling.

## Linux support

| Session | Behavior |
| --- | --- |
| X11 + KWin | Requests blur through `_KDE_NET_WM_BLUR_BEHIND_REGION`. KWin's Blur effect must be enabled. |
| X11 + Picom, including LeftWM | Uses an ARGB window. Picom's configuration determines whether translucent backgrounds are blurred. |
| X11 without a compositor or a 32-bit visual | Keeps normal theme backgrounds and logs why glass is unavailable. Toggle glass again after enabling a compositor. |
| Wayland with `org_kde_kwin_blur_manager` | Requests compositor blur through Winit's `set_blur`. |
| Other Wayland compositors | Translucent theme surfaces; blur depends on compositor support and user rules. |

An accepted blur request is not proof that the desktop renders blur. A compositor
can ignore it or have the effect disabled. See [Winit's blur API](https://docs.rs/winit/0.30.12/winit/window/struct.Window.html#method.set_blur)
and [KDE's X11 implementation](https://github.com/KDE/kwindowsystem/blob/master/src/platforms/xcb/kwindoweffects.cpp).

### Picom / LeftWM

If the background is translucent but sharp, check the configuration used by the
running Picom process. A typical configuration includes:

```conf
backend = "glx";
blur-method = "dual_kawase";
blur-strength = 5;
blur-background = true;
```

Check exclusions or per-window rules if this still does not blur. BasScript's
window class is `BasScript`. Compositor versions differ; use the settings supported
by your installed version. Picom documents the ARGB blur switch in its
[sample configuration](https://github.com/yshui/picom/blob/next/picom.sample.conf).
BasScript does not modify the desktop configuration.

## Renderer integration

The Linux implementation uses Wgpu's native GLES backend, enabled only for Linux
in `app/Cargo.toml`. The verified launch selected Mesa Intel UHD Graphics using
OpenGL 4.6 through the display connection on a machine with both Intel and NVIDIA
GPUs. Direct NVIDIA rendering remains unverified with the final initialization plugin.
This backend is selected at startup even with glass disabled so the switches can
take effect without restarting the app. Windows and macOS retain their existing
renderer selection.

`app/src/linux_renderer.rs` replaces only renderer initialization, at the normal
position of `RenderPlugin` in `DefaultPlugins`:

- Pass Winit's owned display handle to Wgpu's `InstanceDescriptor`. Bevy 0.19's
  automatic initialization passes `None`; Wgpu 29's EGL backend can then choose
  a headless configuration with no presentation modes. Using the same display
  connection is also required for Wayland surfaces.
- Request limits suitable for OpenGL 3.3, retaining the adapter's texture-size
  and buffer-alignment limits. Bevy uses uniform buffers and CPU preprocessing.
- Disable Wgpu's optional indirect-call validation compute shader for GL 3.3
  compatibility. Other validation flags keep their build/environment defaults.
- Hand the device, queue, adapter and instance to Bevy's `RenderCreation::manual`.
  Bevy still owns UI rendering, presentation and the reactive event loop.

See [Wgpu's display requirement](https://docs.rs/wgpu/29.0.4/wgpu/struct.InstanceDescriptor.html#structfield.display).
This is an application integration with Bevy/Wgpu 0.19/29 and should be revisited
on upgrades. Bevy's manual render resources are reused on device recovery; a
lost OpenGL device may require restarting BasScript.

The NVIDIA/X11 Vulkan surface tested during development advertised only `Opaque`.
Requesting `PreMultiplied` caused surface validation to fail. A later automatic
alpha-mode run lost its X11 connection. Linux therefore uses EGL instead of
forcing an unsupported Vulkan alpha mode. Wgpu's EGL backend advertises only
`Opaque` even though its framebuffer blit preserves per-pixel alpha; the window
uses `Auto` and an ARGB visual. No capability checks are bypassed.

## Why earlier attempts could stay opaque

All layers need to preserve transparency:

1. Create an alpha-capable native window. X11 cannot change its visual after
   creation, so Linux now starts with `Window.transparent = true` even when the
   glass switches are off.
2. Use a presentation path that preserves alpha: Linux uses EGL with `Auto`,
   Windows uses `PreMultiplied`, and macOS uses `PostMultiplied`.
3. Clear the camera target with `Color::NONE`.
4. Remove the opaque `WindowSurfaceRoot` background while glass is active.
   Making only a child panel transparent exposes its parent, not the desktop.
5. Apply native preferences on the main thread. Bevy's `WINIT_WINDOWS` registry
   is thread-local; reading it from a worker can find no window.
6. Synchronize UI colors after the native effect's state has been updated.

The implementation caches native preferences so typing and caret movement do
not repeatedly reapply effects or open X11 connections. X11 uses `x11rb` to check
the actual window depth and compositor selection, set KDE's full-window blur
property, and remove it when the last glass switch is disabled. Opaque surfaces
still cover the blur. Wayland uses Winit's existing blur protocol integration.

The working reference in `~/dev/cssimpler/renderer/src/native_glass.rs` similarly
uses an initially transparent window and renderer tint on Linux. Its Linux
native hook does not itself request compositor blur. BasScript adds that request
without changing Bevy, Winit or the CSSimpler checkout.

## Windows and macOS

The existing Windows acrylic / blur fallback and macOS
`NSVisualEffectMaterial::UnderWindowBackground` integrations are retained. The
main-thread and root-background fixes also apply to them; macOS now explicitly
selects its transparent composite mode. Windows retains DX12 and its existing
presentation-system setting.

Linux is the first validation target. Windows and macOS still need native smoke
tests for material appearance, resizing, decorated windows and disabling effects.
Cross-platform support here describes the implemented paths, not a claim that
all three have been visually verified.

## Verification

`cargo test --workspace --locked --offline` passed all 204 tests after this change.
The glass regression tests cover
independent switches, transparent ancestors, removing the paper background,
restoring normal colors when disabled or unavailable, preserving the Canvas
background and retaining a lower Linux theme alpha.

For a native smoke test:

1. Start with the glass switches off and verify the normal editor appearance.
2. Enable Explorer glass, then Processed and Settings glass independently.
3. Place the window over visible desktop content. Check for transparency and
   separately check whether that content is blurred; text should remain crisp.
4. Resize, switch views, open settings and toggle the system titlebar.
5. Disable every glass switch and verify the normal opaque backgrounds return.

On 2026-09-08 the user inspected the rebuilt app and confirmed that the glass
effect worked correctly on Linux/X11 with LeftWM and Picom. The successful log
reported Mesa Intel UHD Graphics (CML GT2), OpenGL 4.6, Mesa 26.2.1. This manual
check used isolated settings with all three glass switches enabled. Native
Wayland, KWin, Windows and macOS validation remains outstanding.

On X11, `xprop` can inspect `WM_CLASS` and `_KDE_NET_WM_BLUR_BEHIND_REGION` on
the BasScript window. The latter is an empty `CARDINAL` property while glass is
active and is removed when the final switch is disabled. It is a request, not
compositor capability detection. Native failures appear under `[window]` in logs.
