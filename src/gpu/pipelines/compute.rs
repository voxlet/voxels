use std::sync::{Arc, Mutex};

use crate::gpu::{shader::Shaders, state, voxel};

const WORKGROUP_SIZE: u32 = 30;

pub struct Compute {
    device: Arc<wgpu::Device>,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    shaders: Arc<Mutex<Shaders>>,
    source_file: String,
}

impl Compute {
    pub fn new(device: Arc<wgpu::Device>, shaders: Arc<Mutex<Shaders>>) -> Self {
        let source_file = "compute.wgsl";
        let compute_shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            flags: wgpu::ShaderFlags::default(),
            source: shaders.lock().unwrap().source(source_file),
        });

        let pixel_buffer_layout_entry = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStage::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Bind Group Layout"),
            entries: &[
                state::bind_group_layout_entry(0, wgpu::ShaderStage::COMPUTE),
                pixel_buffer_layout_entry,
                voxel::texture_layout_entry(2),
                voxel::sampler_layout_entry(3),
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &compute_shader_module,
            entry_point: "main",
        });

        Compute {
            device,
            pipeline,
            bind_group_layout,
            shaders,
            source_file: source_file.to_string(),
        }
    }

    pub fn watch_source<F>(&mut self, mut on_recreate: F)
    where
        F: 'static + FnMut(Compute) + Send,
    {
        let device = self.device.clone();
        let shaders = self.shaders.clone();
        self.shaders
            .lock()
            .unwrap()
            .watch_source(&self.source_file, move || {
                on_recreate(Self::new(device.clone(), shaders.clone()))
            })
    }

    pub fn compute(
        &self,
        device: &wgpu::Device,
        state: &state::State,
        pixel_buffer: &wgpu::Buffer,
        voxel_view: &wgpu::TextureView,
        voxel_sampler: &wgpu::Sampler,
    ) -> wgpu::CommandEncoder {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                state.bind_group_entry(0),
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: pixel_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&voxel_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&voxel_sampler),
                },
            ],
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch(
                state.render_width / WORKGROUP_SIZE,
                state.render_height / WORKGROUP_SIZE,
                1,
            );
        }
        encoder
    }
}
