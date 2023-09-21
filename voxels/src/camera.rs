use std::f32::consts::PI;

use bevy::{input::mouse::MouseMotion, math::Vec2Swizzles, prelude::*};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraControlSettings>()
            .insert_resource(CameraControlSettings {
                rotate_sensitivity: 0.002,
                move_speed: 0.05,
                sprint_factor: 10.0,
            })
            .add_event::<CameraControlEvent>()
            .add_systems(Update, input)
            .add_systems(Update, control);
    }
}

#[derive(Resource, Reflect, Debug, Default)]
pub struct CameraControlSettings {
    pub rotate_sensitivity: f32,
    pub move_speed: f32,
    pub sprint_factor: f32,
}

#[derive(Bundle)]
pub struct CameraBundle {
    camera: Camera3dBundle,
    controller: CameraController,
}

impl CameraBundle {
    pub fn new(eye: Vec3, look_target: Vec3) -> Self {
        let transform = Transform::from_translation(eye).looking_at(look_target, Vec3::Y);

        Self {
            camera: Camera3dBundle {
                transform,
                ..default()
            },
            controller: CameraController {},
        }
    }
}

#[derive(Component, Reflect, Debug)]
struct CameraController {}

#[derive(Event)]
pub struct CameraControlEvent {
    pub delta_rotation: Vec2,
    pub delta_translation: Vec3,
}

fn input(
    settings: Res<CameraControlSettings>,
    keys: Res<Input<KeyCode>>,
    mut mouse_events: EventReader<MouseMotion>,
    mut control_events: EventWriter<CameraControlEvent>,
) {
    let delta_rotation = (mouse_events.iter().fold(Vec2::ZERO, |z, ev| z - ev.delta)
        * settings.rotate_sensitivity)
        .yx();

    let mut delta_translation = keys.get_pressed().fold(Vec3::ZERO, |z, keycode| {
        z + match keycode {
            KeyCode::W => -Vec3::Z,
            KeyCode::A => -Vec3::X,
            KeyCode::S => Vec3::Z,
            KeyCode::D => Vec3::X,
            KeyCode::ControlLeft => -Vec3::Y,
            KeyCode::Space => Vec3::Y,
            _ => Vec3::ZERO,
        }
    });
    if delta_translation != Vec3::ZERO {
        delta_translation = delta_translation.normalize()
            * if keys.pressed(KeyCode::ShiftLeft) {
                settings.move_speed * settings.sprint_factor
            } else {
                settings.move_speed
            };
    }

    if delta_rotation != Vec2::ZERO || delta_translation != Vec3::ZERO {
        control_events.send(CameraControlEvent {
            delta_rotation,
            delta_translation,
        })
    }
}

fn control(
    mut events: EventReader<CameraControlEvent>,
    mut cameras: Query<(&CameraController, &mut Transform)>,
) {
    for ev in events.iter() {
        cameras.for_each_mut(|(_, mut tr)| {
            tr.rotate_y(ev.delta_rotation.y);
            let max_drx = tr.forward().angle_between(Vec3::Y);
            tr.rotate_local_x(ev.delta_rotation.x.clamp(max_drx - PI, max_drx));

            let dtl = tr.rotation * ev.delta_translation;
            tr.translation += dtl;
        })
    }
}
