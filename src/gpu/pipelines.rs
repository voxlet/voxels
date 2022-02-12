mod compute;
pub mod mipmap;
mod render;

use std::sync::{Arc, Mutex};

use compute::Compute;
use render::Render;

use super::shader::Shaders;

pub struct Pipelines {
    pub compute: Arc<Mutex<Compute>>,
    pub render: Render,
}

impl Pipelines {
    pub fn new(
        device: &Arc<wgpu::Device>,
        shaders: &Arc<Mutex<Shaders>>,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let compute = Arc::new(Mutex::new(Compute::new(device.clone(), shaders.clone())));

        let c = compute.clone();
        compute.lock().unwrap().watch_source(move |new_compute| {
            *c.lock().unwrap() = new_compute;
        });

        Pipelines {
            compute,
            render: Render::new(&*device, &mut shaders.lock().unwrap(), surface_config),
        }
    }
}
