use std::{mem::size_of, num::NonZeroU32};

use rayon::prelude::*;

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

fn caves(size: usize) -> Vec<[u8; 4]> {
    tracing::info!("generating cave noise");
    let noise = simdnoise::NoiseBuilder::gradient_3d(size, size, size)
        .with_seed(42)
        .with_freq(10.0 / size as f32)
        .generate_scaled(0.0, 1.0);

    tracing::info!("allocating caves");
    let mut data: Vec<[u8; 4]>;
    let capacity = size * size * size;
    let layout = std::alloc::Layout::array::<[u8; 4]>(capacity).unwrap();
    unsafe {
        data = Vec::from_raw_parts(
            std::alloc::alloc_zeroed(layout) as *mut _,
            capacity,
            capacity,
        );
    }

    tracing::info!("slicing caves");
    let y_offs = size;
    let z_offs = size * size;

    let mut slices: Vec<(&mut [[u8; 4]], usize)> = Vec::with_capacity(size);
    let mut rest = data.as_mut_slice();
    let mut z = 0;
    loop {
        let (head, tail) = rest.split_at_mut(z_offs);
        slices.push((head, z));
        if tail.is_empty() {
            break;
        }
        rest = tail;
        z += 1;
    }

    tracing::info!("digging caves");
    slices.par_iter_mut().for_each(|(slice, z)| {
        for y in 0..size {
            for x in 0..size {
                let i = x + y * y_offs + *z * z_offs;
                let n = noise[i];
                if n > 0.6 {
                    (*slice)[x + y * y_offs] = [
                        to_color(*z, size),
                        to_color(y, size),
                        to_color(x, size),
                        255,
                    ]
                }
            }
        }
    });

    data
}

pub fn create_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    voxel_size: f32,
) -> (wgpu::TextureView, wgpu::Sampler) {
    let size = (1.0 / voxel_size).ceil() as usize;

    //let data = cubic_lattice(SIZE);
    let data = caves(size);

    let extent = wgpu::Extent3d {
        width: size as u32,
        height: size as u32,
        depth_or_array_layers: size as u32,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Voxel Texture"),
        size: extent,
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
            bytes_per_row: NonZeroU32::new(size_of::<u8>() as u32 * 4 * size as u32),
            rows_per_image: NonZeroU32::new(size as u32),
        },
        extent,
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
