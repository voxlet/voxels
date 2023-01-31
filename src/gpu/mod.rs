mod pipelines;
use std::mem::size_of;

pub use pipelines::Pipelines;

pub struct Init {
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub pixel_buffer_desc: wgpu::BufferDescriptor<'static>,
    pub pixel_buffer: wgpu::Buffer,
    pub swap_chain_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub pipelines: Pipelines,
    pub size: winit::dpi::PhysicalSize<u32>,
}

fn pixel_buffer_size(size: &winit::dpi::PhysicalSize<u32>) -> u64 {
    u64::from(size.width * size.height * 4 * size_of::<f32>() as u32)
}

pub async fn init(window: &winit::window::Window) -> Init {
    let size = window.inner_size();

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

    let pixel_buffer_desc = wgpu::BufferDescriptor {
        label: Some("Compute Pixel Buffer"),
        size: pixel_buffer_size(&size),
        usage: wgpu::BufferUsage::STORAGE,
        mapped_at_creation: false,
    };
    let pixel_buffer = device.create_buffer(&pixel_buffer_desc);

    let swap_chain_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

    let pipelines = Pipelines::new(&device, &swap_chain_desc);

    Init {
        surface,
        adapter,
        device,
        queue,
        pixel_buffer_desc,
        pixel_buffer,
        swap_chain_desc,
        swap_chain,
        pipelines,
        size,
    }
}

pub struct Resize {
    pub pixel_buffer: wgpu::Buffer,
    pub swap_chain: wgpu::SwapChain,
}

pub fn resize(
    device: &wgpu::Device,
    surface: &wgpu::Surface,
    new_size: &winit::dpi::PhysicalSize<u32>,
    pixel_buffer_desc: &mut wgpu::BufferDescriptor<'static>,
    swap_chain_desc: &mut wgpu::SwapChainDescriptor,
) -> Resize {
    pixel_buffer_desc.size = pixel_buffer_size(new_size);
    swap_chain_desc.width = new_size.width;
    swap_chain_desc.height = new_size.height;

    let pixel_buffer = device.create_buffer(&pixel_buffer_desc);
    let swap_chain = device.create_swap_chain(surface, &swap_chain_desc);

    Resize {
        pixel_buffer,
        swap_chain,
    }
}
