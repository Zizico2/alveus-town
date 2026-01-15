use bevy::prelude::*;

use crate::components::{PlayerMovementSet, Player};

const CAMERA_DECAY_RATE: f32 = 2.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, update_camera.after(PlayerMovementSet));
    }
}

fn setup_camera(mut commands: Commands, mut window: Single<&mut Window>) {
    window.resolution.set(1080., 1920.);
    commands.spawn(Camera2d);
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
