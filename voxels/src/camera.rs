use bevy::{input::mouse::MouseMotion, math::Vec2Swizzles, prelude::*};
use bevy_inspector_egui::Inspectable;

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
            .add_system(input)
            .add_system(control);
    }
}

#[derive(Reflect, Inspectable, Debug, Default)]
pub struct CameraControlSettings {
    pub rotate_sensitivity: f32,
    pub move_speed: f32,
    pub sprint_factor: f32,
}

#[derive(Bundle)]
pub struct CameraBundle {
    #[bundle]
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

pub struct CameraControlEvent {
    pub rotation_delta: Vec2,
    pub translation_delta: Vec3,
}

fn input(
    settings: Res<CameraControlSettings>,
    keys: Res<Input<KeyCode>>,
    mut mouse_events: EventReader<MouseMotion>,
    mut control_events: EventWriter<CameraControlEvent>,
) {
    let rotation_delta = (mouse_events.iter().fold(Vec2::ZERO, |z, ev| z - ev.delta)
        * settings.rotate_sensitivity)
        .yx();

    let mut translation_delta = keys.get_pressed().fold(Vec3::ZERO, |z, keycode| {
        z + match keycode {
            KeyCode::W => -Vec3::Z,
            KeyCode::A => -Vec3::X,
            KeyCode::S => Vec3::Z,
            KeyCode::D => Vec3::X,
            KeyCode::LControl => -Vec3::Y,
            KeyCode::Space => Vec3::Y,
            _ => Vec3::ZERO,
        }
    });
    if translation_delta != Vec3::ZERO {
        translation_delta = translation_delta.normalize()
            * if keys.pressed(KeyCode::LShift) {
                settings.move_speed * settings.sprint_factor
            } else {
                settings.move_speed
            };
    }

    if rotation_delta != Vec2::ZERO || translation_delta != Vec3::ZERO {
        control_events.send(CameraControlEvent {
            rotation_delta,
            translation_delta,
        })
    }
}

fn control(
    mut events: EventReader<CameraControlEvent>,
    mut cameras: Query<(&CameraController, &mut Transform)>,
) {
    for ev in events.iter() {
        cameras.for_each_mut(|(_, mut tr)| {
            tr.rotate_y(ev.rotation_delta.y);
            tr.rotate_local_x(ev.rotation_delta.x);
            let rotation = tr.rotation;
            tr.translation += rotation * ev.translation_delta;
        })
    }
}
