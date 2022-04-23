use bevy::prelude::*;
use smooth_bevy_cameras::controllers::fps::{FpsCameraBundle, FpsCameraController};

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    #[bundle]
    camera: FpsCameraBundle,
}

const EYE_HEIGHT: f32 = 1.6;

impl PlayerBundle {
    pub fn new(pos: Vec3, look_target: Vec3) -> Self {
        Self {
            player: Player {},
            camera: FpsCameraBundle::new(
                FpsCameraController {
                    smoothing_weight: 0.7,
                    ..FpsCameraController::default()
                },
                PerspectiveCameraBundle::default(),
                pos + Vec3::new(0.0, EYE_HEIGHT, 0.0),
                look_target,
            ),
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct Player;
