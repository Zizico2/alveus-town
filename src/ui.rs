use core::str;
use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{lens::UiPositionLens, *};

use crate::{
    components::{DisplayCurrentTile, Player, PlayerMovementSet, TilePosition},
    entrance::{PlayerEnteredBuildingEvent, PlayerExitedBuildingEvent},
};

pub struct UiPlugin;

pub struct BuildingEnterToast {
    toast_entity: Entity,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_tile_display, spawn_instructions))
            .add_systems(Update, update_current_tile_display.after(PlayerMovementSet));

        // TODO: move this to a separate plugin

        app.add_observer(player_entering_building_observer)
            .add_observer(player_exiting_building_observer);
    }
}

fn player_exiting_building_observer(
    trigger: On<PlayerExitedBuildingEvent>,
    mut commands: Commands,
) {
}

// fn player_entering_building_observer(
//     trigger: On<PlayerEnteredBuildingEvent>,
//     mut commands: Commands,
//     asset_server: ResMut<AssetServer>,
// ) {
//     commands
//         .spawn((Node {
//             position_type: PositionType::Absolute,
//             bottom: px(12),
//             left: px(12),
//             flex_direction: FlexDirection::Column,
//             ..default()
//         },))
//         .with_children(|builder| {
//             builder.spawn((
//                 ImageNode {
//                     image: asset_server.load("enter_building_toast.png"),
//                     ..default()
//                 },
//                 Node {
//                     width: px(150),
//                     ..default()
//                 },
//             ));
//         });
// }

fn player_entering_building_observer(
    trigger: On<PlayerEnteredBuildingEvent>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    // 1. Define the Tween using UiPositionLens
    let slide_up_tween = Tween::new(
        EaseFunction::CubicOut,
        Duration::from_millis(3000),
        UiPositionLens {
            start: UiRect {
                bottom: Val::Px(-150.0),
                left: Val::Px(12.0),
                top: Val::Auto,
                right: Val::Auto,
            },
            end: UiRect {
                bottom: Val::Px(12.0),
                left: Val::Px(12.0),
                top: Val::Auto,
                right: Val::Auto,
            },
        },
    );

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            // The Animator will immediately overwrite these, but good practice to match start
            bottom: Val::Px(-150.0),
            left: Val::Px(12.0),
            // flex_direction: FlexDirection::Column,
            width: Val::Px(150.0),
            height: Val::Px(100.0),
            ..default()
        },
        // 2. Add the Animator
        TweenAnim::new(slide_up_tween),
        ImageNode {
            image: asset_server.load("enter_building_toast.png"),
            ..default()
        },
    ));
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

fn spawn_instructions(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    // commands
    //     .spawn((
    //         // Sprite {
    //         //     custom_size: Some(Vec2::new(300.0, 60.0)),
    //         //     ..Sprite::from_image(asset_server.load("enter_building_toast.png"))
    //         // },
    //         Node {
    //             position_type: PositionType::Absolute,
    //             bottom: px(12),
    //             left: px(12),
    //             flex_direction: FlexDirection::Column,
    //             ..default()
    //         },
    //     ))
    //     .with_children(|builder| {
    //         // builder.spawn(Text::new(
    //         //     "Move the light with WASD.\nThe camera will smoothly track the light.",
    //         // ));
    //         // builder.spawn(Text::new(
    //         //     "Move the light with WASD.\nThe camera will smoothly track the light.",
    //         // ));
    //         builder.spawn((
    //             ImageNode {
    //                 image: asset_server.load("enter_building_toast.png"),
    //                 ..default()
    //             },
    //             Node {
    //                 width: px(150),
    //                 // height: px(60),
    //                 ..default()
    //             },
    //         ));
    //     });
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
