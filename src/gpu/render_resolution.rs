use wgpu::util::DeviceExt;

pub struct RenderResolution {
    pub width: u32,
    pub height: u32,
    pub uniform: wgpu::Buffer,
}

impl RenderResolution {
    pub fn from(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Resolution Uniform Descriptor"),
            contents: bytemuck::cast_slice(&[width as f32, height as f32]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        Self {
            width,
            height,
            uniform,
        }
    }
}
