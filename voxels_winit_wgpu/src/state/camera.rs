use std::{collections::HashSet, f32::consts::PI, time::Duration};

use winit::event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode};

#[derive(PartialEq, Eq, Hash)]
enum CameraMotion {
    Forward,
    Backward,
    Right,
    Left,
    Up,
    Down,
    Sprint,
}

pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub mouse_sensitivity: f32,
    active_motions: HashSet<CameraMotion>,
    pub max_velocity: f32,
    pub position: glam::Vec3,
    pub rotation: glam::Mat3,
}

fn rotation(yaw: f32, pitch: f32) -> glam::Mat3 {
    glam::Mat3::from_euler(glam::EulerRot::YXZ, yaw, pitch, 0.0)
}

impl Camera {
    pub fn new() -> Self {
        let yaw = 3.77;
        let pitch = 0.53;
        let rotation = rotation(yaw, pitch);
        Camera {
            yaw,
            pitch,
            mouse_sensitivity: 0.002,
            active_motions: HashSet::new(),
            position: glam::vec3(0.78, 1.02, 0.83),
            max_velocity: 0.1,
            rotation,
        }
    }

    pub fn input(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                let (x, y) = *delta;
                self.yaw += x as f32 * self.mouse_sensitivity;
                self.yaw = self.yaw.rem_euclid(2.0 * PI);
                self.pitch += y as f32 * self.mouse_sensitivity;
                self.pitch = self.pitch.clamp(-0.5 * PI, 0.5 * PI);
            }
            DeviceEvent::Key(KeyboardInput {
                state,
                virtual_keycode,
                ..
            }) => {
                if let Some(vk) = virtual_keycode {
                    if let Some(motion) = match vk {
                        VirtualKeyCode::W => Some(CameraMotion::Forward),
                        VirtualKeyCode::A => Some(CameraMotion::Left),
                        VirtualKeyCode::S => Some(CameraMotion::Backward),
                        VirtualKeyCode::D => Some(CameraMotion::Right),
                        VirtualKeyCode::Space => Some(CameraMotion::Up),
                        VirtualKeyCode::LControl => Some(CameraMotion::Down),
                        VirtualKeyCode::LShift => Some(CameraMotion::Sprint),
                        _ => None,
                    } {
                        match state {
                            ElementState::Pressed => self.active_motions.insert(motion),
                            ElementState::Released => self.active_motions.remove(&motion),
                        };
                    }
                }
            }
            _ => {
                tracing::debug!(event = ?event, "input");
            }
        }
    }

    pub fn update(&mut self, dt: Duration) {
        let max_velocity = if self.active_motions.contains(&CameraMotion::Sprint) {
            self.max_velocity * 10.0
        } else {
            self.max_velocity
        };
        self.rotation = rotation(self.yaw, self.pitch);

        if let Some(dir) = (&self.active_motions)
            .into_iter()
            .map(|motion| match motion {
                CameraMotion::Forward => glam::vec3(0.0, 0.0, 1.0),
                CameraMotion::Backward => glam::vec3(0.0, 0.0, -1.0),
                CameraMotion::Right => glam::vec3(1.0, 0.0, 0.0),
                CameraMotion::Left => glam::vec3(-1.0, 0.0, 0.0),
                CameraMotion::Up => glam::vec3(0.0, 1.0, 0.0),
                CameraMotion::Down => glam::vec3(0.0, -1.0, 0.0),
                _ => glam::vec3(0.0, 0.0, 0.0),
            })
            .reduce(|z, v| z + v)
        {
            let vel = (self.rotation * dir).clamp_length_max(max_velocity);
            self.position += vel * dt.as_secs_f32();
        }
    }
}
