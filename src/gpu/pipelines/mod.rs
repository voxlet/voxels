mod compute;
mod render;

use compute::Compute;
use render::Render;

use super::shader::Shaders;

pub struct Pipelines {
    pub compute: Compute,
    pub render: Render,
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        shaders: &Shaders,
        swap_chain_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {
        Pipelines {
            compute: Compute::new(device, shaders),
            render: Render::new(device, shaders, swap_chain_desc),
        }
    }
}
