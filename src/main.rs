use bevy::{post_process::bloom::Bloom, prelude::*};
use bevy_ecs_tiled::prelude::*;

/// Player movement speed factor.
const PLAYER_SPEED: f32 = 32.;

/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 2.;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct MovementCooldown(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TiledPlugin::default())
        .add_systems(Startup, (setup_scene, setup_instructions, setup_camera))
        .add_systems(Update, (move_player, update_camera).chain())
        .run();
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let map_handle: Handle<TiledMapAsset> = asset_server.load("map.tmx");
    // World where we move the player
    commands.spawn((
        // Mesh2d(meshes.add(Rectangle::new(1000., 700.))),
        // MeshMaterial2d(materials.add(Color::srgb(0.2, 0.2, 0.3))),
        // Sprite::from_image(asset_server.load("testmap.png")),
        TiledMap(map_handle),
    ));

    // Player
    commands.spawn((
        Player,
        MovementCooldown(Timer::from_seconds(0.2, TimerMode::Once)),
        Mesh2d(meshes.add(Circle::new(16.))),
        // MeshMaterial2d(materials.add(Color::srgb(6.25, 9.4, 9.1))), // RGB values exceed 1 to achieve a bright color for the bloom effect
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.1, 0.9))),
        Transform::from_xyz(0., 0., 2.),
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

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Bloom::NATURAL));
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
    mut player: Single<(&mut Transform, &mut MovementCooldown), With<Player>>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    let (mut transform, mut cooldown) = player.into_inner();

    // 1. Advance the timer by the time elapsed since the last frame
    cooldown.0.tick(time.delta());

    // 2. If the timer hasn't finished (the cooldown is still active), stop here.
    if !cooldown.0.is_finished() {
        return;
    }

    let mut direction = Vec2::ZERO;

    if kb_input.pressed(KeyCode::KeyW) {
        direction.y += 1.;
    } else if kb_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.;
    } else if kb_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    } else if kb_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    // Progressively update the player's position over time. Normalize the
    // direction vector to prevent it from exceeding a magnitude of 1 when
    // moving diagonally.
    // let move_delta = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_secs();

    if direction != Vec2::ZERO {
        let move_delta = direction.normalize_or_zero() * PLAYER_SPEED;
        transform.translation += move_delta.extend(0.);

        // 4. Reset the timer to trigger the 0.5s cooldown
        cooldown.0.reset();
    }
}
