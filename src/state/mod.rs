use winit::{event::WindowEvent, window::Window};

use crate::gpu;

pub struct State {
    pub size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    gpu: gpu::Gpu,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        Self {
            size: window.inner_size(),
            clear_color: wgpu::Color::BLACK,
            gpu: gpu::Gpu::new(window).await,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        tracing::info!(new_size = ?new_size, "State::resize");
        self.size = new_size;
        self.gpu.resize(new_size.width, new_size.height);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        tracing::info!(event = ?event, "State::input");
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.x as f64 / self.size.width as f64,
                    g: position.y as f64 / self.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                };
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {
        // todo!()
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        self.gpu.render()
    }
}
