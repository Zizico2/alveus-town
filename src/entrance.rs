use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::components::{
    BuildingEntrance, Player, PlayerMovementSet, PlayerSetupSet, RectangleTileGroup, TileGroup,
    TilePosition,
};

pub struct EntrancePlugin;

impl Plugin for EntrancePlugin {
    fn build(&self, app: &mut App) {
        // let player = app
        //     .world_mut()
        //     .query_filtered::<Entity, With<Player>>()
        //     .single(app.world())
        //     .expect("TODO: how to handle this error?");

        app.add_systems(Update, validate_and_snap_entrances)
            .add_systems(
                Update,
                (
                    check_player_enter_building,
                    check_player_exit_building,
                    handle_player_entering_building,
                )
                    .chain()
                    .after(PlayerMovementSet),
            )
            .add_systems(
                Startup,
                (|player: Single<Entity, With<Player>>, mut commands: Commands| {
                    commands.entity(*player);
                })
                .after(PlayerSetupSet),
            )
            .add_observer(player_exiting_building_observer);
    }
}

#[derive(Debug, Event)]
pub struct PlayerEnteredBuildingEvent {
    pub entrance: BuildingEntrance,
    // #[event_target]
    // pub player: Entity,
}

#[derive(Debug, Event)]
pub struct PlayerExitedBuildingEvent {
    pub entrance: BuildingEntrance,
    // #[event_target]
    // pub player: Entity,
}

/// Check if the player is within a building entrance and attach the marker component.
fn check_player_enter_building(
    player: Single<
        (&TilePosition, Entity),
        (
            With<Player>,
            Changed<TilePosition>,
            Without<BuildingEntrance>,
        ),
    >,
    entrance: Single<(&TileGroup, &BuildingEntrance)>,
    mut commands: Commands,
) {
    let (player_pos, player_entity) = *player;

    let (entrance_pos, entrance) = *entrance;
    match entrance_pos {
        TileGroup::Rectangle(rect) => {
            if player_pos.x >= rect.bottom_left.x
                && player_pos.x <= rect.top_right.x
                && player_pos.y >= rect.bottom_left.y
                && player_pos.y <= rect.top_right.y
            {
                commands.entity(player_entity).insert(*entrance);
            }
        }
    }
}

fn check_player_exit_building(
    player: Single<
        (&TilePosition, Entity),
        (With<Player>, Changed<TilePosition>, With<BuildingEntrance>),
    >,
    entrances: Query<(&TileGroup, &BuildingEntrance)>,
    mut commands: Commands,
) {
    let (player_pos, player_entity) = *player;

    for (entrance_pos, entrance) in entrances.iter() {
        match entrance_pos {
            TileGroup::Rectangle(rect) => {
                let inside = player_pos.x >= rect.bottom_left.x
                    && player_pos.x <= rect.top_right.x
                    && player_pos.y >= rect.bottom_left.y
                    && player_pos.y <= rect.top_right.y;

                if !inside {
                    commands.trigger(PlayerExitedBuildingEvent {
                        entrance: *entrance,
                        // player: player_entity,
                    });
                }
            }
        }
    }
}

fn handle_player_entering_building(
    player: Single<(&BuildingEntrance, Entity), (With<Player>, Added<BuildingEntrance>)>,
    mut commands: Commands,
) {
    let (entrance, entity) = *player;
    info!("Player entered building: {:?}", entrance);
    commands.trigger(PlayerEnteredBuildingEvent {
        entrance: *entrance,
        // player: entity,
    });
}

fn player_exiting_building_observer(
    trigger: On<PlayerExitedBuildingEvent>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
) {
    let entity = player.entity();
    info!("Player exited building: {:?}", trigger.event().entrance);
    commands.entity(entity).remove::<BuildingEntrance>();
}

fn validate_and_snap_entrances(
    mut commands: Commands,
    query: Query<
        (Entity, &Transform, &BuildingEntrance, &TiledObject),
        (Added<BuildingEntrance>, Without<TileGroup>),
    >,
) {
    const TILE_SIZE: f32 = 32.0;
    const EPSILON: f32 = 0.05;

    for (entity, transform, entrance, tiled_object) in query.iter() {
        let x = transform.translation.x;
        let y = transform.translation.y;

        let rem_x = x.rem_euclid(TILE_SIZE);
        let rem_y = y.rem_euclid(TILE_SIZE);

        let dist_x = rem_x.min(TILE_SIZE - rem_x);
        let dist_y = rem_y.min(TILE_SIZE - rem_y);

        if dist_x >= EPSILON || dist_y >= EPSILON {
            panic!(
                "\n❌ MAP INTEGRITY ERROR ❌\nObject: '{:?}'\nPosition: [x:{:.2}, y:{:.2}]\nIssue: Not aligned to {}-pixel grid.\n",
                entrance, x, y, TILE_SIZE
            );
        }

        let TiledObject::Rectangle { width, height } = tiled_object else {
            panic!(
                "\n❌ MAP INTEGRITY ERROR ❌\nObject: '{:?}'\nIssue: Unsupported TiledObject type for size validation.\n",
                entrance
            );
        };

        if width % TILE_SIZE != 0.0 || height % TILE_SIZE != 0.0 {
            panic!(
                "\n❌ MAP INTEGRITY ERROR ❌\nObject: '{:?}'\nSize: [w:{}, h:{}]\nIssue: Dimensions are not multiples of tile size ({}).\n",
                entrance, width, height, TILE_SIZE
            );
        }

        let adjusted_y = y - height;

        let start_grid_x = (x / TILE_SIZE).round() as u32;
        let start_grid_y = (adjusted_y / TILE_SIZE).round() as u32;

        let width_in_tiles = (width / TILE_SIZE).round() as u32;
        let height_in_tiles = (height / TILE_SIZE).round() as u32;

        let tile_group = TileGroup::Rectangle(RectangleTileGroup {
            bottom_left: TilePosition {
                x: start_grid_x,
                y: start_grid_y,
            },
            top_right: TilePosition {
                x: start_grid_x + width_in_tiles - 1,
                y: start_grid_y + height_in_tiles - 1,
            },
        });
        info!("Inserting TileGroup: {:?}", tile_group);

        commands.entity(entity).insert(tile_group);
    }
}
