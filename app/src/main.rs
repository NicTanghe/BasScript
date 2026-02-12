use basscript_ui::UiPlugin;
use bevy::{asset::AssetPlugin, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "..".to_string(),
            ..default()
        }))
        .add_plugins(UiPlugin)
        .run();
}
