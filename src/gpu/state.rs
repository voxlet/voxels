use std::mem::size_of;

use wgpu::util::DeviceExt;

use crate::state::camera::Camera;

pub struct Data {
    pub camera_rotation: [f32; 9],
    pub camera_position: [f32; 3],
    resolution: [f32; 2],
}

fn concat_slices(slices: &[&[u8]]) -> Vec<u8> {
    let mut vec = Vec::with_capacity(size_of::<Data>());
    slices.iter().for_each(|slice| vec.extend_from_slice(slice));
    vec
}

// align(16), size(12)
fn vec3_bytes(v: &[f32; 3]) -> [u8; 16] {
    let mut buf = [0u8; 16];
    &buf[0..12].copy_from_slice(bytemuck::cast_slice(v));
    buf
}

// align(16), size(48)
fn mat3x3_bytes(m: &[f32; 9]) -> [u8; 48] {
    let mut buf = [0u8; 48];
    &buf[0..12].copy_from_slice(bytemuck::cast_slice(&m[0..3]));
    &buf[16..28].copy_from_slice(bytemuck::cast_slice(&m[3..6]));
    &buf[32..44].copy_from_slice(bytemuck::cast_slice(&m[6..9]));
    buf
}

impl Data {
    fn bytes(&self) -> Vec<u8> {
        concat_slices(&[
            &mat3x3_bytes(&self.camera_rotation),
            &vec3_bytes(&self.camera_position),
            bytemuck::bytes_of(&self.resolution),
        ])
    }
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
    pub fn from(
        device: &wgpu::Device,
        render_width: u32,
        render_height: u32,
        camera: &Camera,
    ) -> Self {
        let data = Data {
            camera_rotation: camera.rotation.to_cols_array(),
            camera_position: camera.position.into(),
            resolution: [render_width as f32, render_height as f32],
        };
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Resolution Uniform Descriptor"),
            contents: bytemuck::cast_slice(&data.bytes()),
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

    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        render_width: Option<u32>,
        render_height: Option<u32>,
        camera: Option<&Camera>,
    ) {
        if let Some(w) = render_width {
            self.render_width = w;
            self.data.resolution[0] = w as f32;
        }
        if let Some(h) = render_height {
            self.render_height = h;
            self.data.resolution[1] = h as f32;
        }
        if let Some(c) = camera {
            self.data.camera_rotation = c.rotation.to_cols_array();
            self.data.camera_position = c.position.into();
        }

        queue.write_buffer(&self.uniform, 0, bytemuck::cast_slice(&self.data.bytes()));
    }
}
