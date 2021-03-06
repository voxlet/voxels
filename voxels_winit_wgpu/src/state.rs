pub mod camera;

use std::time::Duration;

use winit::event::DeviceEvent;

use crate::gpu::voxel;

pub struct State {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub scale_factor: f64,
    pub start_time: std::time::Instant,
    pub last_update_time: std::time::Instant,
    pub physics_step_time: std::time::Duration,
    pub physics_write_time: std::time::Duration,
    pub voxel_transfer_time: std::time::Duration,
    pub dt: std::time::Duration,
    pub voxel_resolution: usize,
    pub voxels: Vec<[u8; 4]>,
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
        let voxel_resolution = 64;
        let camera = camera::Camera::new();
        Self {
            size: window.inner_size(),
            scale_factor: window.scale_factor(),
            start_time,
            last_update_time: start_time,
            physics_step_time: std::time::Duration::new(0, 0),
            physics_write_time: std::time::Duration::new(0, 0),
            voxel_transfer_time: std::time::Duration::new(0, 0),
            voxels: voxel::caves(voxel_resolution),
            voxel_resolution,
            camera,
            dt: Duration::new(0, 0),
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

    pub fn update(&mut self) -> std::time::Duration {
        let now = std::time::Instant::now();
        let dt = now - self.last_update_time;
        self.last_update_time = now;

        self.camera.update(dt);

        self.dt = dt;
        dt
    }
}
