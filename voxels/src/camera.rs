use bevy::prelude::*;
use smooth_bevy_cameras::{
    controllers::fps::{default_input_map, ControlEvent, FpsCameraBundle, FpsCameraController},
    LookAngles, LookTransform, LookTransformPlugin,
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LookTransformPlugin)
            .add_system(control_system)
            .add_event::<ControlEvent>()
            .add_system(default_input_map)
            .add_startup_system(setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(FpsCameraBundle::new(
        FpsCameraController {
            smoothing_weight: 0.7,
            ..FpsCameraController::default()
        },
        PerspectiveCameraBundle::default(),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 1.0),
    ));
}

pub fn control_system(
    mut events: EventReader<ControlEvent>,
    mut cameras: Query<(&FpsCameraController, &mut LookTransform)>,
) {
    let events = events.iter();

    // Can only control one camera at a time.
    let (controller, mut transform) = cameras.single_mut();
    if !controller.enabled {
        return;
    }

    let look_vector = transform.look_direction().unwrap();
    let mut look_angles = LookAngles::from_vector(look_vector);

    let rot = Quat::from_euler(
        EulerRot::YXZ,
        look_angles.get_yaw(),
        -look_angles.get_pitch(),
        0.0,
    );

    for event in events {
        match event {
            ControlEvent::Rotate(delta) => {
                // Rotates with pitch and yaw.
                look_angles.add_yaw(-delta.x);
                look_angles.add_pitch(-delta.y);
            }
            ControlEvent::TranslateEye(delta) => {
                transform.eye += rot * *delta;
            }
        }
    }

    look_angles.assert_not_looking_up();

    transform.target = transform.eye + transform.radius() * look_angles.unit_vector();
}
