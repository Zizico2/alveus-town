use bevy::prelude::*;

use crate::components::{DisplayCurrentTile, GameplaySet, Player, TilePosition};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_tile_display, spawn_instructions))
            .add_systems(Update, update_current_tile_display.in_set(GameplaySet::PostMovement));
    }
}

fn spawn_tile_display(mut commands: Commands) {
    commands.spawn((
        DisplayCurrentTile,
        Text::new("Current Tile Position: (0, 0)"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

fn spawn_instructions(mut commands: Commands) {
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

fn update_current_tile_display(
    player: Single<&TilePosition, With<Player>>,
    mut display: Single<&mut Text, With<DisplayCurrentTile>>,
) {
    **display = Text::new(format!(
        "Current Tile Position: ({}, {})",
        player.x, player.y
    ));
}
