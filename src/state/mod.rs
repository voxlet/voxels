use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};

use crate::gpu;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    resolution_uniform: wgpu::Buffer,
    pixel_buffer_desc: wgpu::BufferDescriptor<'static>,
    pixel_buffer: wgpu::Buffer,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    pipelines: gpu::Pipelines,
}

fn create_resolution_uniform(
    device: &wgpu::Device,
    size: winit::dpi::PhysicalSize<u32>,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Resolution Uniform Descriptor"),
        contents: bytemuck::cast_slice(&[size.width as f32, size.height as f32]),
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    })
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let gpu::Init {
            surface,
            device,
            queue,
            pixel_buffer_desc,
            pixel_buffer,
            swap_chain_desc,
            swap_chain,
            pipelines,
            size,
            ..
        } = gpu::init(window).await;

        let resolution_uniform = create_resolution_uniform(&device, window.inner_size());

        Self {
            surface,
            device,
            queue,
            resolution_uniform,
            pixel_buffer_desc,
            pixel_buffer,
            swap_chain_desc,
            swap_chain,
            size,
            pipelines,
            clear_color: wgpu::Color::BLACK,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        tracing::info!(new_size = ?new_size, "State::resize");

        let resolution_uniform = create_resolution_uniform(&self.device, new_size);
        let resized = gpu::resize(
            &self.device,
            &self.surface,
            &new_size,
            &mut self.pixel_buffer_desc,
            &mut self.swap_chain_desc,
        );

        self.size = new_size;
        self.resolution_uniform = resolution_uniform;
        self.pixel_buffer = resized.pixel_buffer;
        self.swap_chain = resized.swap_chain;
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
        let frame = self.swap_chain.get_current_frame()?.output;

        let compute_encoder = self.pipelines.compute.compute(
            &self.device,
            &self.pixel_buffer,
            &self.resolution_uniform,
            &self.size,
        );
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(compute_encoder.finish()));

        let render_encoder = self.pipelines.render.render(
            &self.device,
            &frame,
            &self.pixel_buffer,
            &self.resolution_uniform,
        );
        self.queue.submit(std::iter::once(render_encoder.finish()));

        Ok(())
    }
}
