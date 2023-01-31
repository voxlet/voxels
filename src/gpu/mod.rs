mod pipelines;
mod state;

use state::State;
use std::{mem::size_of, num::NonZeroU32};

use crate::state::camera::Camera;
pub use pipelines::Pipelines;

pub struct Gpu {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    state: State,
    voxel_view: wgpu::TextureView,
    voxel_sampler: wgpu::Sampler,
    pixel_buffer_desc: wgpu::BufferDescriptor<'static>,
    pixel_buffer: wgpu::Buffer,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pipelines: Pipelines,
}

#[allow(dead_code)]
fn cubic_lattice(size: usize) -> Vec<[u8; 4]> {
    let mut data = vec![[0u8, 0, 0, 0]; size * size * size];
    let y_offs = size;
    let z_offs = size * size;
    let range = (32..size - 32).step_by(8);
    for z in range.clone() {
        for y in range.clone() {
            for x in range.clone() {
                data[x + y * y_offs + z * z_offs] = [z as u8, y as u8, x as u8, 255];
            }
        }
    }
    data
}

fn caves(size: usize) -> Vec<[u8; 4]> {
    tracing::info!("generating caves");
    let noise = simdnoise::NoiseBuilder::gradient_3d(size, size, size)
        .with_freq(0.03)
        .generate_scaled(0.0, 1.0);
    let mut data = vec![[0u8, 0, 0, 0]; size * size * size];
    let y_offs = size;
    let z_offs = size * size;
    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let i = x + y * y_offs + z * z_offs;
                let n = noise[i];
                if n > 0.6 {
                    data[i] = [z as u8, y as u8, x as u8, 255];
                }
            }
        }
    }

    data
}

fn create_voxel_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> (wgpu::TextureView, wgpu::Sampler) {
    const SIZE: usize = 256;

    //let data = cubic_lattice(SIZE);
    let data = caves(SIZE);

    let size = wgpu::Extent3d {
        width: SIZE as u32,
        height: SIZE as u32,
        depth_or_array_layers: SIZE as u32,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Voxel Texture"),
        size,
        mip_level_count: 3,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("Voxel View"),
        ..wgpu::TextureViewDescriptor::default()
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        bytemuck::cast_slice(&data),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(size_of::<u8>() as u32 * 4 * SIZE as u32),
            rows_per_image: NonZeroU32::new(SIZE as u32),
        },
        size,
    );

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Voxel Sampler"),
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..wgpu::SamplerDescriptor::default()
    });

    (view, sampler)
}

fn pixel_buffer_size(width: u32, height: u32) -> u64 {
    u64::from(width * height * 4 * size_of::<f32>() as u32)
}

impl Gpu {
    pub async fn new(window: &winit::window::Window, camera: &Camera) -> Self {
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
        let state = State::from(&device, width, height, camera);

        let (voxel_view, voxel_sampler) = create_voxel_texture(&device, &queue);

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
            width: state.render_width,
            height: state.render_height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let pipelines = Pipelines::new(&device, &swap_chain_desc);

        Gpu {
            surface,
            device,
            queue,
            state,
            voxel_view,
            voxel_sampler,
            pixel_buffer_desc,
            pixel_buffer,
            swap_chain_desc,
            swap_chain,
            pipelines,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.state
            .update(&self.queue, Some(width), Some(height), None);

        self.pixel_buffer_desc.size = pixel_buffer_size(width, height);
        self.pixel_buffer = self.device.create_buffer(&self.pixel_buffer_desc);

        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    pub fn update(&mut self, _dt: std::time::Duration, camera: &Camera) {
        self.state.update(&self.queue, None, None, Some(camera));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let compute_encoder = self.pipelines.compute.compute(
            &self.device,
            &self.state,
            &self.pixel_buffer,
            &self.voxel_view,
            &self.voxel_sampler,
        );
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(compute_encoder.finish()));

        let render_encoder =
            self.pipelines
                .render
                .render(&self.device, &self.state, &frame, &self.pixel_buffer);
        self.queue.submit(std::iter::once(render_encoder.finish()));

        Ok(())
    }
}
