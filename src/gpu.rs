mod pipelines;
pub mod shader;
mod state;
pub mod voxel;

use state::State;
use std::{
    mem::size_of,
    sync::{Arc, Mutex},
};

use crate::state::camera::Camera;
pub use pipelines::Pipelines;

use shader::Shaders;

pub struct Gpu {
    surface: wgpu::Surface,
    pub device: Arc<wgpu::Device>,
    pub queue: wgpu::Queue,
    pub state: State,
    voxel_texture_desc: wgpu::TextureDescriptor<'static>,
    voxel_texture: wgpu::Texture,
    voxel_view: wgpu::TextureView,
    voxel_nearest_sampler: wgpu::Sampler,
    voxel_linear_sampler: wgpu::Sampler,
    pixel_buffer_desc: wgpu::BufferDescriptor<'static>,
    pixel_buffer: wgpu::Buffer,
    pub swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    shaders: Arc<Mutex<Shaders>>,
    pipelines: Pipelines,
}

fn pixel_buffer_size(width: u32, height: u32) -> u64 {
    u64::from(width * height * 4 * size_of::<f32>() as u32)
}

impl Gpu {
    pub async fn new(
        window: &winit::window::Window,
        voxels: &Vec<[u8; 4]>,
        voxel_resolution: usize,
        camera: &Camera,
    ) -> Self {
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
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

        device.on_uncaptured_error(|error| tracing::error!(?error));

        let winit::dpi::PhysicalSize { width, height } = window.inner_size();
        let state = State::from(&device, width, height, camera);

        let mut shaders = Shaders::new("shaders");
        let (
            voxel_texture,
            voxel_texture_desc,
            voxel_view,
            voxel_nearest_sampler,
            voxel_linear_sampler,
        ) = voxel::create_texture(&device, &queue, &mut shaders, voxels, voxel_resolution);

        let pixel_buffer_desc = wgpu::BufferDescriptor {
            label: Some("Compute Pixel Buffer"),
            size: pixel_buffer_size(width, height),
            usage: wgpu::BufferUsage::STORAGE,
            mapped_at_creation: false,
        };
        let pixel_buffer = device.create_buffer(&pixel_buffer_desc);

        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: state.render_width,
            height: state.render_height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let device = Arc::new(device);
        let shaders = Arc::new(Mutex::new(shaders));
        let pipelines = Pipelines::new(&device, &shaders, &swap_chain_desc);

        Gpu {
            surface,
            device,
            queue,
            state,
            voxel_texture,
            voxel_texture_desc,
            voxel_view,
            voxel_nearest_sampler,
            voxel_linear_sampler,
            pixel_buffer_desc,
            pixel_buffer,
            swap_chain_desc,
            swap_chain,
            shaders,
            pipelines,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.pixel_buffer_desc.size = pixel_buffer_size(width, height);
        self.pixel_buffer = self.device.create_buffer(&self.pixel_buffer_desc);

        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_desc);

        self.state.update(
            &self.queue,
            &state::Update {
                render_width: Some(width),
                render_height: Some(height),
                ..Default::default()
            },
        );
    }

    pub fn update(&mut self, _dt: std::time::Duration, camera: &Camera) {
        self.state.update(
            &self.queue,
            &state::Update {
                camera: Some(camera),
                ..Default::default()
            },
        );
    }

    pub fn set_voxels(&mut self, voxels: &Vec<[u8; 4]>, voxel_resolution: usize) {
        let (
            voxel_texture,
            voxel_texture_desc,
            voxel_view,
            voxel_nearest_sampler,
            voxel_linear_sampler,
        ) = voxel::create_texture(
            &self.device,
            &self.queue,
            &mut self.shaders.lock().unwrap(),
            voxels,
            voxel_resolution,
        );

        self.voxel_texture = voxel_texture;
        self.voxel_texture_desc = voxel_texture_desc;
        self.voxel_view = voxel_view;
        self.voxel_nearest_sampler = voxel_nearest_sampler;
        self.voxel_linear_sampler = voxel_linear_sampler;

        self.state.update(
            &self.queue,
            &state::Update {
                voxel_size: Some(1.0 / voxel_resolution as f32),
                ..Default::default()
            },
        );
    }

    pub fn update_voxels(&mut self, voxels: &Vec<[u8; 4]>, size: usize) {
        voxel::update_texture(
            &self.device,
            &self.queue,
            &mut self.shaders.lock().unwrap(),
            &self.voxel_texture,
            &self.voxel_texture_desc,
            voxels,
            size,
        )
    }

    pub fn get_current_frame(&self) -> Result<wgpu::SwapChainTexture, wgpu::SwapChainError> {
        Ok(self.swap_chain.get_current_frame()?.output)
    }

    pub fn render(&mut self, frame: &wgpu::SwapChainTexture) -> wgpu::CommandEncoder {
        let compute_encoder = self.pipelines.compute.lock().unwrap().compute(
            &self.device,
            &self.state,
            &self.pixel_buffer,
            &self.voxel_view,
            &self.voxel_nearest_sampler,
            &self.voxel_linear_sampler,
        );
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(compute_encoder.finish()));

        self.pipelines
            .render
            .render(&self.device, &self.state, frame, &self.pixel_buffer)
    }
}
