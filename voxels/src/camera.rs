use bevy::prelude::*;
use smooth_bevy_cameras::controllers::fps as smooth_fps_camera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(smooth_bevy_cameras::LookTransformPlugin)
            .add_plugin(smooth_fps_camera::FpsCameraPlugin::default())
            .add_startup_system(setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(smooth_fps_camera::FpsCameraBundle::new(
        smooth_fps_camera::FpsCameraController {
            smoothing_weight: 0.5,
            ..smooth_fps_camera::FpsCameraController::default()
        },
        PerspectiveCameraBundle::default(),
        Vec3::new(80.0, 80.0, 300.0),
        Vec3::new(0., 0., 0.),
    ));
}
