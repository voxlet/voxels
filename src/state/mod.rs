pub mod camera;

use std::time::Duration;

use winit::event::DeviceEvent;

pub struct State {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub scale_factor: f64,
    pub start_time: std::time::Instant,
    pub last_update_time: std::time::Instant,
    pub camera: camera::Camera,
}

pub fn grab_cursor(window: &winit::window::Window, grab: bool) {
    let _ = window.set_cursor_grab(grab).unwrap();
    window.set_cursor_visible(!grab);
}

impl State {
    pub fn new(window: &winit::window::Window) -> Self {
        grab_cursor(window, true);

        let start_time = std::time::Instant::now();
        let camera = camera::Camera::new();
        Self {
            size: window.inner_size(),
            scale_factor: window.scale_factor(),
            start_time,
            last_update_time: start_time,
            camera,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, scale_factor: f64) {
        self.size = new_size;
        self.scale_factor = scale_factor;
    }

    pub fn input(&mut self, _window: &winit::window::Window, event: &DeviceEvent) {
        match event {
            _ => self.camera.input(event),
        };
    }

    pub fn update(&mut self) -> Duration {
        let now = std::time::Instant::now();
        let dt = now - self.last_update_time;
        self.last_update_time = now;

        self.camera.update(dt);

        dt
    }
}
