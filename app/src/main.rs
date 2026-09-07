use basscript_ui::UiPlugin;
#[cfg(target_os = "windows")]
use bevy::render::{
    RenderPlugin,
    settings::{Backends, WgpuSettings},
};
#[cfg(target_os = "windows")]
use bevy::window::CompositeAlphaMode;
use bevy::{
    asset::AssetPlugin,
    log::LogPlugin,
    prelude::*,
    window::{WindowPlugin, WindowResizeConstraints},
};

const MIN_WINDOW_WIDTH: f32 = 640.0;
const MIN_WINDOW_HEIGHT: f32 = 360.0;
const NATIVE_TRANSPARENT_WINDOW: bool = cfg!(any(target_os = "windows", target_os = "macos"));
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

    let default_plugins = DefaultPlugins
        .set(LogPlugin {
            filter: format!(
                "{},bevy_render::view::window=error",
                bevy::log::DEFAULT_FILTER
            ),
            ..default()
        })
        .set(AssetPlugin {
            file_path: "..".to_string(),
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                decorations: false,
                transparent: NATIVE_TRANSPARENT_WINDOW,
                resize_constraints: WindowResizeConstraints {
                    min_width: MIN_WINDOW_WIDTH,
                    min_height: MIN_WINDOW_HEIGHT,
                    ..default()
                },
                #[cfg(target_os = "windows")]
                composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
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

    App::new()
        .insert_resource(ClearColor(if NATIVE_TRANSPARENT_WINDOW {
            Color::NONE
        } else {
            Color::srgb(0.89, 0.90, 0.91)
        }))
        .add_plugins(default_plugins)
        // Bevy 0.18 used a small local bevy_winit patch to avoid transient
        // below-minimum resize events desynchronizing UI layout from the window.
        // Keep that behavior in app code so 0.19 can use upstream winit/wgpu.
        .add_systems(First, clamp_window_resolution_to_constraints)
        .add_plugins(UiPlugin)
        .run();
}

fn clamp_window_resolution_to_constraints(mut windows: Query<&mut Window, Changed<Window>>) {
    for mut window in &mut windows {
        let size = constrained_window_physical_size(&window);

        if size.x != window.resolution.physical_width()
            || size.y != window.resolution.physical_height()
        {
            window.resolution.set_physical_resolution(size.x, size.y);
        }
    }
}

fn constrained_window_physical_size(window: &Window) -> UVec2 {
    let constraints = window.resize_constraints.check_constraints();
    let scale_factor = window.resolution.scale_factor().max(f32::EPSILON);
    let min_width = (constraints.min_width * scale_factor).ceil().max(1.0) as u32;
    let min_height = (constraints.min_height * scale_factor).ceil().max(1.0) as u32;

    UVec2::new(
        window.resolution.physical_width().max(min_width),
        window.resolution.physical_height().max(min_height),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_a_resize_to_the_configured_minimum() {
        let mut window = Window {
            resize_constraints: WindowResizeConstraints {
                min_width: MIN_WINDOW_WIDTH,
                min_height: MIN_WINDOW_HEIGHT,
                ..default()
            },
            ..default()
        };
        window.resolution.set_scale_factor(1.5);
        window.resolution.set_physical_resolution(800, 400);

        assert_eq!(
            constrained_window_physical_size(&window),
            UVec2::new(960, 540)
        );
    }
}
