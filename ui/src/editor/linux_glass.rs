//! Linux desktop glass: ARGB transparency plus a compositor blur request.

use winit::{
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::Window,
};
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ConnectionExt, PropMode},
    wrapper::ConnectionExt as _,
};

const BLUR_BEHIND_REGION: &[u8] = b"_KDE_NET_WM_BLUR_BEHIND_REGION";

/// Called on Bevy's main thread, with the window still alive.
/// `true` means transparency is available, not that the compositor promises blur.
pub(super) fn apply(window: &Window, enabled: bool) -> Result<bool, String> {
    let handle = window.window_handle().map_err(|error| error.to_string())?;
    match handle.as_raw() {
        RawWindowHandle::Wayland(_) => {
            // Winit uses org_kde_kwin_blur_manager when advertised. Other
            // compositors keep ordinary translucency or use user-defined rules.
            window.set_blur(enabled);
            Ok(enabled)
        }
        RawWindowHandle::Xlib(handle) => {
            let window_id = u32::try_from(handle.window).map_err(|error| error.to_string())?;
            apply_x11(window_id, enabled).map_err(|error| error.to_string())
        }
        RawWindowHandle::Xcb(handle) => {
            apply_x11(handle.window.get(), enabled).map_err(|error| error.to_string())
        }
        _ => Ok(false),
    }
}

fn apply_x11(window: u32, enabled: bool) -> Result<bool, Box<dyn std::error::Error>> {
    // Only reached when native preferences change. A separate connection avoids
    // taking ownership of Winit's Xlib/XCB connection or calling Xlib unsafely.
    let (connection, _) = x11rb::connect(None)?;
    let blur_atom = connection
        .intern_atom(false, BLUR_BEHIND_REGION)?
        .reply()?
        .atom;

    if !enabled {
        connection.delete_property(window, blur_atom)?.check()?;
        return Ok(false);
    }

    let geometry = connection.get_geometry(window)?.reply()?;
    if geometry.depth != 32 {
        return Err("the X11 window has no 32-bit ARGB visual".into());
    }
    let screen = connection
        .setup()
        .roots
        .iter()
        .position(|screen| screen.root == geometry.root)
        .ok_or("the X11 window's screen was not found")?;
    let selection = connection
        .intern_atom(false, format!("_NET_WM_CM_S{screen}").as_bytes())?
        .reply()?
        .atom;
    if connection.get_selection_owner(selection)?.reply()?.owner == x11rb::NONE {
        return Err(
            "no X11 compositor is running; enable a compositor and toggle glass again".into(),
        );
    }

    // KDE's empty CARDINAL region means the entire window and follows resizes.
    // Opaque UI pixels still cover the effect. Picom may ignore this hint and
    // instead blur ARGB windows according to its own blur-background rules.
    connection
        .change_property32(
            PropMode::REPLACE,
            window,
            blur_atom,
            AtomEnum::CARDINAL,
            &[],
        )?
        .check()?;
    Ok(true)
}
