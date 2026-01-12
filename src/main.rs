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

#[derive(Component, Debug)]
struct TilePositionArray {
    positions: Vec<TilePosition>,
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
        .add_systems(Update, validate_and_snap_entrances)
        // Observers for specific events
        // .add_observer(on_add_building_entrance)
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
        .spawn((
            TiledMap(asset_server.load("map.tmx")),
            TilemapAnchor::BottomLeft,
        ))
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
        transform.translation.x = (tile_position.x * TILE_SIZE) as f32 + TILE_SIZE as f32 / 2.0;
        transform.translation.y = (tile_position.y * TILE_SIZE) as f32 + TILE_SIZE as f32 / 2.0;
    }
}

// --- Observers / Events ---

// fn on_add_building_entrance(trigger: On<Add, BuildingEntrance>) {
//     info!("Added BuildingEntrance component: {:?}", trigger.event());
// }
fn validate_and_snap_entrances(
    mut commands: Commands,
    // Added TiledObject to get width/height.
    // If your size data is in 'BuildingEntrance', access it there instead.
    query: Query<
        (Entity, &Transform, &BuildingEntrance, &TiledObject),
        (Added<BuildingEntrance>, Without<TilePositionArray>),
    >,
) {
    const TILE_SIZE: f32 = 32.0;
    const EPSILON: f32 = 0.05;

    for (entity, transform, entrance, tiled_object) in query.iter() {
        let x = transform.translation.x;
        let y = transform.translation.y;

        // 1. Validate Alignment (Origin)
        // ----------------------------------------------------
        let rem_x = x.rem_euclid(TILE_SIZE);
        let rem_y = y.rem_euclid(TILE_SIZE);

        let dist_x = rem_x.min(TILE_SIZE - rem_x);
        let dist_y = rem_y.min(TILE_SIZE - rem_y);

        if dist_x >= EPSILON || dist_y >= EPSILON {
            panic!(
                "\n❌ MAP INTEGRITY ERROR ❌\nObject: '{}'\nPosition: [x:{:.2}, y:{:.2}]\nIssue: Not aligned to {}-pixel grid.\n",
                entrance.building_entrance, x, y, TILE_SIZE
            );
        }

        // 2. Validate Dimensions (Must be multiples of tile size)
        // ----------------------------------------------------
        // Tiled objects usually store size in pixels
        let TiledObject::Rectangle { width, height } = tiled_object else {
            panic!(
                "\n❌ MAP INTEGRITY ERROR ❌\nObject: '{}'\nIssue: Unsupported TiledObject type for size validation.\n",
                entrance.building_entrance
            );
        };

        if width % TILE_SIZE != 0.0 || height % TILE_SIZE != 0.0 {
            panic!(
                "\n❌ MAP INTEGRITY ERROR ❌\nObject: '{}'\nSize: [w:{}, h:{}]\nIssue: Dimensions are not multiples of tile size ({}).\n",
                entrance.building_entrance, width, height, TILE_SIZE
            );
        }

        // 3. Calculate All Occupied Tiles
        // ----------------------------------------------------
        // Fix for "as": Use round() instead of floor().
        // If x is 31.99 (valid by epsilon), floor makes it 31 (wrong), round makes it 32 (correct).
        let start_grid_x = (x / TILE_SIZE).round() as u32;
        let start_grid_y = (y / TILE_SIZE).round() as u32;

        let width_in_tiles = (width / TILE_SIZE).round() as u32;
        let height_in_tiles = (height / TILE_SIZE).round() as u32;

        let mut positions = Vec::with_capacity((width_in_tiles * height_in_tiles) as usize);

        // Iterate through the dimensions to fill the vector
        // NOTE: Tiled Y-coordinates can be tricky (Up vs Down).
        // Standard 2D loops usually look like this:
        for dx in 0..width_in_tiles {
            for dy in 0..height_in_tiles {
                positions.push(TilePosition {
                    x: start_grid_x + dx,
                    // Depending on your coordinate system (Y-Up vs Y-Down),
                    // you might need `start_grid_y - dy` here.
                    // Bevy is usually Y-Up, Tiled is Y-Down. Check your map loader settings.
                    y: start_grid_y + dy,
                });
            }
        }

        info!(
            "Snapping entrance '{}' to {} tiles starting at [{}, {}]",
            entrance.building_entrance,
            positions.len(),
            start_grid_x,
            start_grid_y
        );

        // 4. Store the Array
        commands
            .entity(entity)
            .insert(TilePositionArray { positions });
    }
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
