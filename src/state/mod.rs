pub mod camera;

use winit::event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode};

use crate::gpu;

pub struct State {
    pub size: winit::dpi::PhysicalSize<u32>,
    _start_time: std::time::Instant,
    last_update_time: std::time::Instant,
    gpu: gpu::Gpu,
    camera: camera::Camera,
}

fn grab_cursor(window: &winit::window::Window, grab: bool) {
    let _ = window.set_cursor_grab(grab).unwrap();
    window.set_cursor_visible(!grab);
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &winit::window::Window) -> Self {
        let _start_time = std::time::Instant::now();
        let camera = camera::Camera::new();
        let gpu = gpu::Gpu::new(window, &camera).await;
        Self {
            size: window.inner_size(),
            _start_time,
            last_update_time: _start_time,
            camera,
            gpu,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.gpu.resize(new_size.width, new_size.height);
    }

    pub fn input(&mut self, window: &winit::window::Window, event: &DeviceEvent) {
        match event {
            DeviceEvent::Button {
                state: ElementState::Released,
                ..
            } => {
                grab_cursor(window, true);
            }

            DeviceEvent::Key(KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Escape),
                ..
            }) => {
                grab_cursor(window, false);
            }

            _ => self.camera.input(event),
        };
    }

    pub fn update(&mut self) {
        let now = std::time::Instant::now();
        let dt = now - self.last_update_time;
        self.camera.update(dt);
        self.gpu.update(dt, &self.camera);
        self.last_update_time = now;
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        self.gpu.render()
    }
}
