use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use std::env;

// --- Constants ---
const TILE_SIZE: u32 = 32;
const CAMERA_DECAY_RATE: f32 = 2.0;
const PLAYER_Z_INDEX: f32 = 2.0;

// --- Components ---

#[derive(Component)]
struct Player;

#[derive(Component, Clone, Copy, Debug)]
struct TilePosition {
    x: u32,
    y: u32,
}

#[derive(Component)]
struct MovementCooldown(Timer);

#[derive(Component)]
struct DisplayCurrentTile;

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
struct BuildingEntrance {
    pub building_entrance: String,
}

// --- Main ---

fn main() {
    // Construct path to tiled types export
    let tiled_types_path = env::current_dir()
        .unwrap()
        .join("assets")
        .join("tiled_types.json");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TiledPlugin(TiledPluginConfig {
            tiled_types_export_file: Some(tiled_types_path),
            // Filter out internal Bevy components to keep the Tiled export clean
            tiled_types_filter: TiledFilter::from(
                regex::RegexSet::new([r"^alveus_idle::.*"]).unwrap(),
            ),
        }))
        .register_type::<BuildingEntrance>()
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
        // Observers for specific events
        .add_observer(on_add_building_entrance)
        .run();
}

// --- Setup Systems ---

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn Map
    commands
        .spawn(TiledMap(asset_server.load("map.tmx")))
        .observe(on_map_created);

    // Spawn Player
    let initial_tile_position = TilePosition { x: 0, y: 0 };

    commands.spawn((
        Player,
        initial_tile_position,
        MovementCooldown(Timer::from_seconds(0.2, TimerMode::Once)),
        Mesh2d(meshes.add(Circle::new(16.))),
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.1, 0.9))),
        Transform::from_xyz(0., 0., PLAYER_Z_INDEX),
    ));

    // Spawn UI Debug Text
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

fn setup_camera(mut commands: Commands, mut window: Single<&mut Window>) {
    // Set resolution (Portrait mode)
    window.resolution.set(1080., 1920.);
    commands.spawn(Camera2d);
}

// --- Logic Systems ---

fn update_current_tile_display(
    player: Single<&TilePosition, With<Player>>,
    mut display: Single<&mut Text, With<DisplayCurrentTile>>,
) {
    **display = Text::new(format!(
        "Current Tile Position: ({}, {})",
        player.x, player.y
    ));
}

/// Smoothly tracks the player with the camera.
fn update_camera(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let target = Vec3::new(
        player.translation.x,
        player.translation.y,
        camera.translation.z,
    );

    camera
        .translation
        .smooth_nudge(&target, CAMERA_DECAY_RATE, time.delta_secs());
}

/// Simple grid-based movement logic.
fn move_player(
    player: Single<(&mut TilePosition, &mut MovementCooldown), With<Player>>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    let (mut tile_position, mut cooldown) = player.into_inner();

    cooldown.0.tick(time.delta());

    if !cooldown.0.is_finished() {
        return;
    }

    let mut moved = false;

    // Use if/else if to prevent diagonal movement in a single frame
    if kb_input.pressed(KeyCode::KeyW) {
        tile_position.y = tile_position.y.saturating_add(1);
        moved = true;
    } else if kb_input.pressed(KeyCode::KeyS) {
        tile_position.y = tile_position.y.saturating_sub(1);
        moved = true;
    } else if kb_input.pressed(KeyCode::KeyA) {
        tile_position.x = tile_position.x.saturating_sub(1);
        moved = true;
    } else if kb_input.pressed(KeyCode::KeyD) {
        tile_position.x = tile_position.x.saturating_add(1);
        moved = true;
    }

    if moved {
        cooldown.0.reset();
    }
}

/// Syncs the Transform (pixel position) to the TilePosition (grid position).
fn update_player_transform(
    mut query: Query<(&TilePosition, &mut Transform), (With<Player>, Changed<TilePosition>)>,
) {
    for (tile_position, mut transform) in query.iter_mut() {
        transform.translation.x = (tile_position.x * TILE_SIZE) as f32;
        transform.translation.y = (tile_position.y * TILE_SIZE) as f32;
    }
}

// --- Observers / Events ---

fn on_add_building_entrance(trigger: On<Add, BuildingEntrance>) {
    info!("Added BuildingEntrance component: {:?}", trigger.event());
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
