use bevy::prelude::*;

pub const TILE_SIZE: u32 = 32;
pub const PLAYER_Z_INDEX: f32 = 2.0;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PlayerMovementSet;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PlayerSetupSet;

#[derive(Component)]
pub struct Player;

#[derive(Component, Clone, Copy, Debug)]
pub struct TilePosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Component, Debug)]
pub enum TileGroup {
    Rectangle(RectangleTileGroup),
}

#[derive(Component, Debug)]
pub struct RectangleTileGroup {
    pub bottom_left: TilePosition,
    pub top_right: TilePosition,
}

#[derive(Component)]
pub struct MovementCooldown(pub Timer);

#[derive(Component)]
pub struct DisplayCurrentTile;

#[derive(Component, Debug, Reflect, Default, Clone, Copy)]
#[reflect(Component, Default)]
pub enum BuildingEntrance {
    #[default]
    NoEntrance,
    NutritionHouse,
}
