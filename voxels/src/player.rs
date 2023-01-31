use bevy::prelude::*;

use crate::camera::CameraBundle;

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    #[bundle]
    camera: CameraBundle,
}

const EYE_HEIGHT: f32 = 1.6;

impl PlayerBundle {
    pub fn new(pos: Vec3, look_target: Vec3) -> Self {
        Self {
            player: Player,
            camera: CameraBundle::new(pos + Vec3::new(0.0, EYE_HEIGHT, 0.0), look_target, 0.7),
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct Player;
