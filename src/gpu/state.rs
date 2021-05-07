use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Data {
    resolution: [f32; 2],
}

pub fn bind_group_layout_entry(
    binding: u32,
    visibility: wgpu::ShaderStage,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub struct State {
    pub data: Data,
    pub uniform: wgpu::Buffer,
    pub render_width: u32,
    pub render_height: u32,
}

impl State {
    pub fn from(device: &wgpu::Device, render_width: u32, render_height: u32) -> Self {
        let data = Data {
            resolution: [render_width as f32, render_height as f32],
        };
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Resolution Uniform Descriptor"),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        Self {
            data,
            uniform,
            render_width,
            render_height,
        }
    }

    pub fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: self.uniform.as_entire_binding(),
        }
    }
}
