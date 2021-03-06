use voxels_winit_wgpu::app;

fn main() {
    tracing_subscriber::fmt::init();

    tracing::info!("starting");

    futures_lite::future::block_on(app::run());
}
