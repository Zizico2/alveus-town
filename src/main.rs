use std::env;

use bevy::{post_process::bloom::Bloom, prelude::*};
use bevy_ecs_tiled::prelude::*;

/// Player movement speed factor.
const TILE_SIZE: u32 = 32;

/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 2.;

#[derive(Component)]
struct Player;

#[derive(Component, Clone, Copy)]
struct TilePosition {
    x: u32,
    y: u32,
}

#[derive(Component)]
struct MovementCooldown(Timer);

fn main() {
    let mut path = env::current_dir().unwrap();
    path.push("assets");
    path.push("tiled_types.json");

    App::new()
        .register_type::<BuildingEntrance>()
        .add_plugins(DefaultPlugins)
        .add_plugins(TiledPlugin(TiledPluginConfig {
            tiled_types_export_file: Some(path),
            tiled_types_filter: TiledFilter::from(
                regex::RegexSet::new([
                    r"^alveus_idle::.*",
                    // r"^bevy_sprite::text2d::Text2d$",
                    // r"^bevy_text::text::TextColor$",
                    // r"^bevy_ecs::name::Name$",
                ])
                .unwrap(),
            ),
        }))
        .add_systems(
            Startup,
            (setup_scene, setup_instructions, setup_camera).chain(),
        )
        .add_systems(
            Update,
            (
                move_player,
                update_current_tile_display,
                update_player_transform,
                update_camera,
            )
                .chain(),
        )
        .add_observer(on_add_building_entrance)
        .run();
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
struct BuildingEntrance {
    pub building_entrance: String,
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let map_handle: Handle<TiledMapAsset> = asset_server.load("map.tmx");
    // World where we move the player
    commands
        .spawn((
            // Mesh2d(meshes.add(Rectangle::new(1000., 700.))),
            // MeshMaterial2d(materials.add(Color::srgb(0.2, 0.2, 0.3))),
            // Sprite::from_image(asset_server.load("testmap.png")),
            TiledMap(map_handle),
        ))
        .observe(on_map_created);

    let initial_tile_position = TilePosition { x: 0, y: 0 };
    // Player
    commands.spawn((
        Player,
        initial_tile_position.clone(),
        MovementCooldown(Timer::from_seconds(0.2, TimerMode::Once)),
        Mesh2d(meshes.add(Circle::new(16.))),
        // MeshMaterial2d(materials.add(Color::srgb(6.25, 9.4, 9.1))), // RGB values exceed 1 to achieve a bright color for the bloom effect
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.1, 0.9))),
        Transform::from_xyz(0., 0., 2.),
    ));

    commands.spawn((
        DisplayCurrentTile,
        Text::new(format!(
            "Current Tile Position: ({}, {})",
            initial_tile_position.x, initial_tile_position.y
        )),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

fn setup_instructions(mut commands: Commands) {
    commands.spawn((
        Text::new("Move the light with WASD.\nThe camera will smoothly track the light."),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12),
            left: px(12),
            ..default()
        },
    ));
}

#[derive(Component)]
struct DisplayCurrentTile;

fn update_current_tile_display(
    player: Single<&TilePosition, With<Player>>,
    display: Single<&mut Text, With<DisplayCurrentTile>>,
) {
    let tile_position = player.into_inner();
    let mut display = display.into_inner();
    *display = Text::new(format!(
        "Current Tile Position: ({}, {})",
        tile_position.x, tile_position.y
    ));
}

fn setup_camera(mut commands: Commands, mut window: Single<&mut Window>) {
    // commands.spawn((Camera2d, Bloom::NATURAL));
    window.resolution.set(1080., 1920.0);
    commands.spawn(Camera2d);
}

/// Update the camera position by tracking the player.
fn update_camera(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    // Applies a smooth effect to camera movement using stable interpolation
    // between the camera position and the player position on the x and y axes.
    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}

/// Update the player position with keyboard inputs.
/// Note that the approach used here is for demonstration purposes only,
/// as the point of this example is to showcase the camera tracking feature.
///
/// A more robust solution for player movement can be found in `examples/movement/physics_in_fixed_timestep.rs`.
fn move_player(
    player: Single<(&mut TilePosition, &mut MovementCooldown), With<Player>>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    let (mut tile_position, mut cooldown) = player.into_inner();

    // 1. Advance the timer by the time elapsed since the last frame
    cooldown.0.tick(time.delta());

    // TODO: once I add animations, this should check if the movement animation is finished instead of the timer
    if !cooldown.0.is_finished() {
        return;
    }

    if kb_input.pressed(KeyCode::KeyW) {
        tile_position.y = tile_position.y.saturating_add(1);
        cooldown.0.reset();
    } else if kb_input.pressed(KeyCode::KeyS) {
        tile_position.y = tile_position.y.saturating_sub(1);
        cooldown.0.reset();
    } else if kb_input.pressed(KeyCode::KeyA) {
        tile_position.x = tile_position.x.saturating_sub(1);
        cooldown.0.reset();
    } else if kb_input.pressed(KeyCode::KeyD) {
        tile_position.x = tile_position.x.saturating_add(1);
        cooldown.0.reset();
    }
}

fn update_player_transform(
    mut query: Query<(&TilePosition, &mut Transform), (With<Player>, Changed<TilePosition>)>,
) {
    for (tile_position, mut transform) in query.iter_mut() {
        // TODO: use fallible conversions
        transform.translation.x = (tile_position.x * TILE_SIZE) as f32;
        transform.translation.y = (tile_position.y * TILE_SIZE) as f32;
    }
}

fn on_add_building_entrance(add_building_entrance: On<Add, BuildingEntrance>) {
    // log
    info!(
        "Added BuildingEntrance component: {:?}",
        add_building_entrance.event()
    );
}

// fn on_tile_created(
//     tile_created: On<TiledEvent<TileCreated>>,

// ) {
//     let tile_entity = tile_created.event().origin;
//     info!("Tile Created: {:?}", tile_entity);
//     info!("Tile Created Event: {:?}", tile_created.event());
// }

fn on_map_created(
    map_created: On<TiledEvent<MapCreated>>,
    map_query: Query<&TiledMapStorage, With<TiledMap>>,
    tiles_query: Query<(&TilePos, Option<&BuildingEntrance>)>,
) {
    // Get the map entity and storage component
    let map_entity = map_created.event().origin;
    let Ok(map_storage) = map_query.get(map_entity) else {
        return;
    };

    // We will iterate over all tiles from our map and try to access our custom properties
    for ((_tile_id, _tileset_id), entities_list) in map_storage.tiles() {
        for tile_entity in entities_list {
            let Ok((pos, building_entrance)) = tiles_query.get(*tile_entity) else {
                continue;
            };

            // Here, we only print the content of our tile but we could also do some
            // global initialization.
            // A typical use case would be to initialize a resource so we can map a tile
            // position to a biome and / or a resource (which could be useful for pathfinding)

            if let Some(i) = building_entrance {
                // Only print the first tile to avoid flooding the console
                info_once!("Found Building Entrance [{:?} @ {:?}]", i, pos);
            }
        }
    }
}
