use basscript_ui::UiPlugin;
use bevy::{
    asset::AssetPlugin,
    prelude::*,
    window::WindowPlugin,
};
#[cfg(target_os = "windows")]
use bevy::window::CompositeAlphaMode;
#[cfg(target_os = "windows")]
use bevy::render::{
    RenderPlugin,
    settings::{Backends, WgpuSettings},
};

fn main() {
    #[cfg(target_os = "windows")]
    if std::env::var_os("WGPU_DX12_PRESENTATION_SYSTEM").is_none() {
        unsafe {
            std::env::set_var("WGPU_DX12_PRESENTATION_SYSTEM", "Visual");
        }
    }

    let default_plugins = DefaultPlugins
        .set(AssetPlugin {
            file_path: "..".to_string(),
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                decorations: false,
                transparent: true,
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
        .insert_resource(ClearColor(Color::NONE))
        .add_plugins(default_plugins)
        .add_plugins(UiPlugin)
        .run();
}

