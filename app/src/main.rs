#[cfg(target_os = "linux")]
mod linux_renderer;
mod reactive_rendering;

use basscript_ui::UiPlugin;
#[cfg(target_os = "windows")]
use bevy::render::{
    RenderPlugin,
    settings::{Backends, WgpuSettings},
};
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use bevy::window::CompositeAlphaMode;
use bevy::{
    asset::AssetPlugin,
    log::LogPlugin,
    prelude::*,
    window::{WindowPlugin, WindowResizeConstraints},
};
use reactive_rendering::ReactiveRenderingPlugin;

const MIN_WINDOW_WIDTH: f32 = 640.0;
const MIN_WINDOW_HEIGHT: f32 = 360.0;
const NATIVE_TRANSPARENT_WINDOW: bool = cfg!(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux"
));
#[cfg(target_os = "linux")]
const LINUX_MIN_THREAD_STACK_BYTES: &str = "8388608";

fn main() {
    #[cfg(target_os = "linux")]
    if std::env::var_os("RUST_MIN_STACK").is_none() {
        // Bevy's render task pool can overflow the default thread stack on some Linux drivers.
        unsafe {
            std::env::set_var("RUST_MIN_STACK", LINUX_MIN_THREAD_STACK_BYTES);
        }
    }

    #[cfg(target_os = "windows")]
    if std::env::var_os("WGPU_DX12_PRESENTATION_SYSTEM").is_none() {
        unsafe {
            std::env::set_var("WGPU_DX12_PRESENTATION_SYSTEM", "Visual");
        }
    }

    configure_asset_root();

    let default_plugins = DefaultPlugins
        .set(LogPlugin {
            filter: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| bevy::log::DEFAULT_FILTER.to_string()),
            ..default()
        })
        .set(AssetPlugin {
            file_path: "".to_string(),
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                name: Some("BasScript".into()),
                decorations: false,
                // X11 needs an ARGB visual at creation, even if glass starts disabled.
                transparent: NATIVE_TRANSPARENT_WINDOW,
                resize_constraints: WindowResizeConstraints {
                    min_width: MIN_WINDOW_WIDTH,
                    min_height: MIN_WINDOW_HEIGHT,
                    ..default()
                },
                #[cfg(target_os = "windows")]
                composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
                #[cfg(target_os = "linux")]
                // Wgpu's EGL backend exposes Auto/Opaque, while its framebuffer
                // blit preserves per-pixel alpha in the native ARGB window.
                composite_alpha_mode: CompositeAlphaMode::Auto,
                #[cfg(target_os = "macos")]
                composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                ..default()
            }),
            ..default()
        });
    #[cfg(target_os = "windows")]
    let default_plugins = default_plugins.set(RenderPlugin {
        render_creation: WgpuSettings {
            backends: Some(Backends::DX12),
            ..default()
        }
        .into(),
        ..default()
    });

    #[cfg(target_os = "linux")]
    let default_plugins = default_plugins
        .add_before::<bevy::render::RenderPlugin>(linux_renderer::LinuxRendererPlugin)
        .disable::<bevy::render::RenderPlugin>();

    App::new()
        .insert_resource(ClearColor(if NATIVE_TRANSPARENT_WINDOW {
            Color::NONE
        } else {
            Color::srgb(0.89, 0.90, 0.91)
        }))
        .add_plugins(default_plugins)
        .add_plugins((UiPlugin, ReactiveRenderingPlugin))
        .run();
}

fn configure_asset_root() {
    if std::env::var_os("BEVY_ASSET_ROOT").is_some() {
        return;
    }
    if std::path::Path::new("fonts").exists()
        && let Ok(cwd) = std::env::current_dir()
    {
        unsafe {
            std::env::set_var("BEVY_ASSET_ROOT", cwd);
        }
        return;
    }
    if std::path::Path::new("../fonts").exists()
        && let Ok(cwd) = std::env::current_dir()
        && let Some(parent) = cwd.parent()
    {
        unsafe {
            std::env::set_var("BEVY_ASSET_ROOT", parent);
        }
        return;
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent();
        while let Some(dir) = cur {
            if dir.join("fonts").exists() {
                unsafe {
                    std::env::set_var("BEVY_ASSET_ROOT", dir);
                }
                return;
            }
            cur = dir.parent();
        }
    }
}
