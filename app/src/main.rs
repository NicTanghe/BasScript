use basscript_ui::UiPlugin;
use bevy::{asset::AssetPlugin, prelude::*, window::WindowPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::NONE))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: "..".to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        decorations: false,
                        transparent: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(UiPlugin)
        .run();
}

