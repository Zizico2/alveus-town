use std::path::PathBuf;

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{regex::RegexSet, *};

use crate::components::BuildingEntrance;

pub struct MapPlugin {
    tiled_types_path: PathBuf,
}

impl MapPlugin {
    pub fn new(path: PathBuf) -> Self {
        Self {
            tiled_types_path: path,
        }
    }
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TiledPlugin(TiledPluginConfig {
            tiled_types_export_file: Some(self.tiled_types_path.clone()),
            // Filter out internal Bevy components to keep the Tiled export clean
            // tiled_types_filter: TiledFilter::Names(vec![
            //     "alveus_idle::components::BuildingEntrance".into(),
            // ]),
            tiled_types_filter: TiledFilter::from(
                RegexSet::new([r"^alveus_idle::components::.*"]).unwrap(),
            ),
        }))
        .add_systems(Startup, spawn_map)
        .add_observer(on_map_created);
    }
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap(asset_server.load("map.tmx")),
        TilemapAnchor::BottomLeft,
    ));
}

/// Post-process map initialization (finding specific tiles, etc).
fn on_map_created(
    trigger: On<TiledEvent<MapCreated>>,
    map_query: Query<&TiledMapStorage, With<TiledMap>>,
    tiles_query: Query<(&TilePos, Option<&BuildingEntrance>)>,
) {
    let map_entity = trigger.event().origin;
    let Ok(map_storage) = map_query.get(map_entity) else {
        return;
    };

    for (_, entities_list) in map_storage.tiles() {
        for &tile_entity in entities_list {
            let Ok((pos, building_entrance)) = tiles_query.get(tile_entity) else {
                continue;
            };

            if let Some(entrance) = building_entrance {
                info_once!("Found Building Entrance [{:?} @ {:?}]", entrance, pos);
            }
        }
    }
}
