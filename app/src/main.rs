use basscript_ui::UiPlugin;
use bevy::{asset::AssetPlugin, prelude::*, window::WindowPlugin};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: "..".to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        decorations: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(UiPlugin)
        .run();
}
