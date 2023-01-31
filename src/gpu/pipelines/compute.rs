use crate::gpu::{shader::Shaders, state};

const WORKGROUP_SIZE: u32 = 30;

pub struct Compute {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Compute {
    pub fn new(device: &wgpu::Device, shaders: &Shaders) -> Self {
        let compute_shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            flags: wgpu::ShaderFlags::default(),
            source: shaders.source("compute.wgsl"),
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

        let voxel_texture_layout_entry = wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStage::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D3,
                multisampled: false,
            },
            count: None,
        };

        let voxel_sampler_layout_entry = wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStage::COMPUTE,
            ty: wgpu::BindingType::Sampler {
                filtering: true,
                comparison: false,
            },
            count: None,
        };

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Bind Group Layout"),
            entries: &[
                state::bind_group_layout_entry(0, wgpu::ShaderStage::COMPUTE),
                pixel_buffer_layout_entry,
                voxel_texture_layout_entry,
                voxel_sampler_layout_entry,
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
            pipeline,
            bind_group_layout,
        }
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