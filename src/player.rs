use bevy::prelude::*;

use crate::components::{
    GameplaySet, MovementCooldown, PLAYER_Z_INDEX, Player, PlayerSetupSet, TILE_SIZE, TilePosition,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player.in_set(PlayerSetupSet))
            .add_systems(
                Update,
                (move_player, update_player_transform)
                    .chain()
                    .in_set(GameplaySet::Movement),
            );
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let initial_tile_position = TilePosition { x: 0, y: 0 };

    commands.spawn((
        Player,
        initial_tile_position,
        MovementCooldown(Timer::from_seconds(0.2, TimerMode::Once)),
        Mesh2d(meshes.add(Circle::new(16.))),
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.1, 0.9))),
        Transform::from_xyz(0., 0., PLAYER_Z_INDEX),
    ));
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
