mod compute;
mod render;

use compute::Compute;
use render::Render;

pub struct Pipelines {
    pub compute: Compute,
    pub render: Render,
}

impl Pipelines {
    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor) -> Self {
        Pipelines {
            compute: Compute::new(device),
            render: Render::new(device, swap_chain_desc),
        }
    }
}
