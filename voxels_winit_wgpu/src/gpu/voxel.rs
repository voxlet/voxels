use std::{mem::size_of, num::NonZeroU32};

use rayon::prelude::*;

use super::{pipelines::mipmap, shader::Shaders};

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

fn to_color(i: usize, size: usize) -> u8 {
    (i * 256 / size) as u8
}

pub fn mut_layers(voxels: &mut Vec<[u8; 4]>, size: usize) -> Vec<&mut [[u8; 4]]> {
    let z_offs = size * size;

    let mut layers: Vec<&mut [[u8; 4]]> = Vec::with_capacity(size);
    let mut rest = voxels.as_mut_slice();
    loop {
        let (head, tail) = rest.split_at_mut(z_offs);
        layers.push(head);
        if tail.is_empty() {
            break;
        }
        rest = tail;
    }

    layers
}

pub fn layers(voxels: &Vec<[u8; 4]>, size: usize) -> Vec<&[[u8; 4]]> {
    let z_offs = size * size;

    let mut layers: Vec<&[[u8; 4]]> = Vec::with_capacity(size);
    let mut rest = voxels.as_slice();
    loop {
        let (head, tail) = rest.split_at(z_offs);
        layers.push(head);
        if tail.is_empty() {
            break;
        }
        rest = tail;
    }

    layers
}

pub fn new_zero_buf(size: usize) -> Vec<[u8; 4]> {
    let capacity = size * size * size;
    let layout = std::alloc::Layout::array::<[u8; 4]>(capacity).unwrap();
    unsafe {
        Vec::from_raw_parts(
            std::alloc::alloc_zeroed(layout) as *mut _,
            capacity,
            capacity,
        )
    }
}

pub fn caves(size: usize) -> Vec<[u8; 4]> {
    tracing::info!("generating cave noise");
    let noise = simdnoise::NoiseBuilder::fbm_3d(size, size, size)
        .with_seed(42)
        .with_freq(14.0 / size as f32)
        .generate_scaled(0.0, 1.0);

    tracing::info!("allocating caves");
    let mut data: Vec<[u8; 4]> = new_zero_buf(size);

    let y_offs = size;
    let z_offs = size * size;

    tracing::info!("digging caves");
    mut_layers(&mut data, size)
        .par_iter_mut()
        .enumerate()
        .for_each(|(z, slice)| {
            for y in 0..size {
                for x in 0..size {
                    let i = x + y * y_offs + z * z_offs;
                    let n = noise[i];
                    if n > 0.6 {
                        (*slice)[x + y * y_offs] =
                            [to_color(z, size), to_color(y, size), to_color(x, size), 255]
                    }
                }
            }
        });

    data
}

pub fn to_size(voxel_size: f32) -> usize {
    (1.0 / voxel_size).ceil() as usize
}

pub fn update_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &mut Shaders,
    texture: &wgpu::Texture,
    texture_desc: &wgpu::TextureDescriptor,
    data: &Vec<[u8; 4]>,
    size: usize,
) {
    let extent = wgpu::Extent3d {
        width: size as u32,
        height: size as u32,
        depth_or_array_layers: size as u32,
    };

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(&data),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(size_of::<u8>() as u32 * 4 * size as u32),
            rows_per_image: NonZeroU32::new(size as u32),
        },
        extent,
    );

    mipmap::generate(device, queue, shaders, &texture_desc, &texture);
}

pub fn create_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &mut Shaders,
    data: &Vec<[u8; 4]>,
    size: usize,
) -> (
    wgpu::Texture,
    wgpu::TextureDescriptor<'static>,
    wgpu::TextureView,
    wgpu::Sampler,
    wgpu::Sampler,
) {
    let extent = wgpu::Extent3d {
        width: size as u32,
        height: size as u32,
        depth_or_array_layers: size as u32,
    };

    let texture_desc = wgpu::TextureDescriptor {
        label: Some("Voxel Texture"),
        size: extent,
        mip_level_count: (size as f32).log2() as u32,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST,
    };
    let texture = device.create_texture(&texture_desc);

    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("Voxel View"),
        ..wgpu::TextureViewDescriptor::default()
    });

    update_texture(device, queue, shaders, &texture, &texture_desc, data, size);

    let nearest_descriptor = wgpu::SamplerDescriptor {
        label: Some("Voxel Nearest Sampler"),
        address_mode_u: wgpu::AddressMode::MirrorRepeat,
        address_mode_v: wgpu::AddressMode::MirrorRepeat,
        address_mode_w: wgpu::AddressMode::MirrorRepeat,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..wgpu::SamplerDescriptor::default()
    };

    let nearest_sampler = device.create_sampler(&nearest_descriptor);

    let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Voxel Linear Sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..nearest_descriptor
    });

    (texture, texture_desc, view, nearest_sampler, linear_sampler)
}

pub fn texture_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D3,
            multisampled: false,
        },
        count: None,
    }
}

pub fn sampler_layout_entry(binding: u32, filtering: bool) -> wgpu::BindGroupLayoutEntry {
    let sampler_binding_type = if filtering {
        wgpu::SamplerBindingType::Filtering
    } else {
        wgpu::SamplerBindingType::NonFiltering
    };

    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Sampler(sampler_binding_type),
        count: None,
    }
}
