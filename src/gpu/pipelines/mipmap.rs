use std::{
    collections::HashMap,
    num::NonZeroU32,
    sync::{Arc, Mutex},
};

use lazy_static::lazy_static;
use wgpu::util::DeviceExt;

use crate::gpu::shader::Shaders;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Args {
    size: u32,
    mip_level: u32,
}

fn args_uniform(device: &wgpu::Device, size: u32, mip_level: u32) -> Arc<wgpu::Buffer> {
    lazy_static! {
        static ref UNIFORMS: Mutex<HashMap<(u32, u32), Arc<wgpu::Buffer>>> =
            Mutex::new(HashMap::new());
    }
    let mut uniforms = UNIFORMS.lock().unwrap();
    if let Some(uniform) = uniforms.get(&(size, mip_level)) {
        return uniform.clone();
    }

    let args = Args { size, mip_level };
    let uniform = Arc::new(
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mipmap Args Uniform"),
            contents: bytemuck::cast_slice(&[args]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        }),
    );

    uniforms.insert((size, mip_level), uniform.clone());
    return uniform;
}

pub fn generate_level(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::ComputePipeline,
    size: u32,
    mip_level: u32,
    texture: &wgpu::Texture,
) {
    let uniform = args_uniform(device, size, mip_level);

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Mipmap Encoder"),
    });

    let input = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("Mipmap Input Texture"),
        base_mip_level: mip_level - 1,
        mip_level_count: NonZeroU32::new(1),
        ..Default::default()
    });

    let output = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("Mipmap Input Texture"),
        base_mip_level: mip_level,
        mip_level_count: NonZeroU32::new(1),
        ..Default::default()
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Mipmap Bind Group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&input),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&output),
            },
        ],
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Mipmap Compute Pass"),
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch(
            size / WORKGROUP_SIZE,
            size / WORKGROUP_SIZE,
            size / WORKGROUP_SIZE,
        );
    }

    queue.submit(std::iter::once(encoder.finish()));
}

const WORKGROUP_SIZE: u32 = 2;

pub fn generate(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &mut Shaders,
    texture_desc: &wgpu::TextureDescriptor,
    texture: &wgpu::Texture,
) {
    let wgpu::TextureDescriptor {
        size,
        mip_level_count,
        ..
    } = texture_desc;

    assert!(
        size.width == size.height && size.height == size.depth_or_array_layers,
        "non-uniform texture"
    );
    assert!(size.width.count_ones() == 1, "non-power-of-2 sized texture");
    let size = size.width as f32;
    assert!(
        size.log2() == *mip_level_count as f32,
        "bad mip level count {}, expected {}",
        mip_level_count,
        size.log2()
    );
    let size = size as u32;

    let source_file = "mipmap.wgsl";
    let compute_shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Mipmap Shader"),
        flags: wgpu::ShaderFlags::default(),
        source: shaders.source(source_file),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Mipmap Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadOnly,
                    format: texture_desc.format,
                    view_dimension: wgpu::TextureViewDimension::D3,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: texture_desc.format,
                    view_dimension: wgpu::TextureViewDimension::D3,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Mipmap Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Mipmap Pipeline"),
        layout: Some(&pipeline_layout),
        module: &compute_shader_module,
        entry_point: "main",
    });

    for mip_level in 1..*mip_level_count {
        generate_level(
            device,
            queue,
            &pipeline,
            size / (2 as u32).pow(mip_level),
            mip_level,
            texture,
        );
    }
}
