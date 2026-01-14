mod camera;
mod components;
mod entrance;
mod map;
mod player;
mod ui;

use bevy::prelude::*;
use std::env;

use components::GameplaySet;
use map::MapPlugin;

fn main() {
    let tiled_types_path = env::current_dir()
        .unwrap()
        .join("assets")
        .join("tiled_types.json");

    App::new()
        .add_plugins(DefaultPlugins)
        .register_type::<components::BuildingEntrance>()
        .configure_sets(
            Update,
            (GameplaySet::Movement, GameplaySet::PostMovement).chain(),
        )
        .add_plugins(MapPlugin::new(tiled_types_path))
        .add_plugins(player::PlayerPlugin)
        .add_plugins((entrance::EntrancePlugin, camera::CameraPlugin, ui::UiPlugin))
        .run();
}
