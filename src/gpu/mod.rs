mod pipelines;
mod render_resolution;

use render_resolution::RenderResolution;
use std::mem::size_of;

pub use pipelines::Pipelines;

pub struct Gpu {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    resolution: RenderResolution,
    pixel_buffer_desc: wgpu::BufferDescriptor<'static>,
    pixel_buffer: wgpu::Buffer,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pipelines: Pipelines,
}

fn pixel_buffer_size(width: u32, height: u32) -> u64 {
    u64::from(width * height * 4 * size_of::<f32>() as u32)
}

impl Gpu {
    pub async fn new(window: &winit::window::Window) -> Self {
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let winit::dpi::PhysicalSize { width, height } = window.inner_size();
        let resolution = RenderResolution::from(&device, width, height);

        let pixel_buffer_desc = wgpu::BufferDescriptor {
            label: Some("Compute Pixel Buffer"),
            size: pixel_buffer_size(width, height),
            usage: wgpu::BufferUsage::STORAGE,
            mapped_at_creation: false,
        };
        let pixel_buffer = device.create_buffer(&pixel_buffer_desc);

        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: resolution.width,
            height: resolution.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let pipelines = Pipelines::new(&device, &swap_chain_desc);

        Gpu {
            surface,
            device,
            queue,
            resolution,
            pixel_buffer_desc,
            pixel_buffer,
            swap_chain_desc,
            swap_chain,
            pipelines,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.resolution = RenderResolution::from(&self.device, width, height);

        self.pixel_buffer_desc.size = pixel_buffer_size(width, height);
        self.pixel_buffer = self.device.create_buffer(&self.pixel_buffer_desc);

        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    pub fn render(&self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let compute_encoder =
            self.pipelines
                .compute
                .compute(&self.device, &self.pixel_buffer, &self.resolution);
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(compute_encoder.finish()));

        let render_encoder = self.pipelines.render.render(
            &self.device,
            &frame,
            &self.pixel_buffer,
            &self.resolution,
        );
        self.queue.submit(std::iter::once(render_encoder.finish()));

        Ok(())
    }
}
