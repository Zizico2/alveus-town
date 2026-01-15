mod camera;
mod components;
mod entrance;
mod map;
mod player;
mod ui;

use bevy::prelude::*;
use serde_json::Value;
use std::{env, fs::File, io::BufReader};

use map::MapPlugin;

fn main() {
    let tiled_types_file =
        File::open("assets/tiled_types.json").expect("Failed to open tiled_types.json");
    let tiled_project = File::open("assets/alveus-idle.tiled-project")
        .expect("Failed to open alveus-idle.tiled-project");
    let mut json: Value = serde_json::from_reader(BufReader::new(tiled_project))
        .expect("Failed to parse alveus-idle.tiled-project as JSON");
    json["propertyTypes"] = serde_json::from_reader(BufReader::new(tiled_types_file))
        .expect("Failed to parse tiled_types.json as JSON");
    let mut output_file = File::create("assets/alveus-idle.tiled-project")
        .expect("Failed to create alveus-idle.tiled-project");
    serde_json::to_writer_pretty(&mut output_file, &json)
        .expect("Failed to write updated alveus-idle.tiled-project");

    let tiled_types_path = env::current_dir()
        .unwrap()
        .join("assets")
        .join("tiled_types.json");

    App::new()
        .add_plugins(DefaultPlugins)
        .register_type::<components::BuildingEntrance>()
        .add_plugins(MapPlugin::new(tiled_types_path))
        .add_plugins(player::PlayerPlugin)
        .add_plugins((entrance::EntrancePlugin, camera::CameraPlugin, ui::UiPlugin))
        .run();
}
