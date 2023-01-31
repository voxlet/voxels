mod app;
mod gpu;
mod state;
mod ui;

fn main() {
    tracing_subscriber::fmt::init();

    tracing::info!("starting");

    futures_lite::future::block_on(app::run());
}
