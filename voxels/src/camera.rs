use bevy::{input::mouse::MouseMotion, prelude::*};
use smooth_bevy_cameras::{
    controllers::fps::{ControlEvent, FpsCameraController},
    LookAngles, LookTransform, LookTransformPlugin,
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LookTransformPlugin)
            .add_event::<ControlEvent>()
            .add_system(control_system)
            .add_system(input_map);
    }
}

pub fn input_map(
    mut events: EventWriter<ControlEvent>,
    keyboard: Res<Input<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    controllers: Query<&FpsCameraController>,
) {
    // Can only control one camera at a time.
    let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
        controller
    } else {
        return;
    };
    let FpsCameraController {
        translate_sensitivity,
        mouse_rotate_sensitivity,
        ..
    } = *controller;

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }

    if cursor_delta != Vec2::ZERO {
        events.send(ControlEvent::Rotate(
            mouse_rotate_sensitivity * cursor_delta,
        ));
    }

    for (key, dir) in [
        (KeyCode::W, Vec3::Z),
        (KeyCode::A, Vec3::X),
        (KeyCode::S, -Vec3::Z),
        (KeyCode::D, -Vec3::X),
        (KeyCode::LShift, -Vec3::Y),
        (KeyCode::Space, Vec3::Y),
    ]
    .iter()
    .cloned()
    {
        if keyboard.pressed(key) {
            events.send(ControlEvent::TranslateEye(translate_sensitivity * dir));
        }
    }
}

pub fn control_system(
    mut events: EventReader<ControlEvent>,
    mut cameras: Query<(&FpsCameraController, &mut LookTransform)>,
) {
    let events = events.iter();

    for event in events {
        // Can only control one camera at a time.
        let (controller, mut transform) = cameras.single_mut();
        if !controller.enabled {
            return;
        }
        let look_vector = transform.look_direction().unwrap();
        let mut look_angles = LookAngles::from_vector(look_vector);

        match event {
            ControlEvent::Rotate(delta) => {
                // Rotates with pitch and yaw.
                look_angles.add_yaw(-delta.x);
                look_angles.add_pitch(-delta.y);
            }
            ControlEvent::TranslateEye(delta) => {
                let rot = Quat::from_euler(
                    EulerRot::YXZ,
                    look_angles.get_yaw(),
                    -look_angles.get_pitch(),
                    0.0,
                );
                transform.eye += rot * *delta;
            }
        }
        look_angles.assert_not_looking_up();
        transform.target = transform.eye + transform.radius() * look_angles.unit_vector();
    }
}
