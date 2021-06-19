mod window;

use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use wgpu::SwapChainTexture;
use winit::{
    event_loop::{EventLoop, EventLoopProxy},
    window::Window,
};

use crate::{gpu::Gpu, physics::Physics, state::State};

pub enum AppEvent {
    RequestRedraw,
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
pub struct RepaintSignal(Mutex<EventLoopProxy<AppEvent>>);

impl epi::RepaintSignal for RepaintSignal {
    fn request_repaint(&self) {
        self.0
            .lock()
            .unwrap()
            .send_event(AppEvent::RequestRedraw)
            .ok();
    }
}

pub struct Ui {
    repaint_signal: Arc<RepaintSignal>,
    pub platform: Platform,
    render_pass: RenderPass,
    start_time: Instant,
    previous_frame_time: Option<f32>,
}

pub fn repaint_signal(event_loop: &EventLoop<AppEvent>) -> Arc<RepaintSignal> {
    Arc::new(RepaintSignal(Mutex::new(event_loop.create_proxy())))
}

impl Ui {
    pub fn new(
        window: &Window,
        repaint_signal: Arc<RepaintSignal>,
        state: &State,
        gpu: &Gpu,
    ) -> Self {
        let platform = Platform::new(PlatformDescriptor {
            physical_width: state.size.width as u32,
            physical_height: state.size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        Ui {
            repaint_signal,
            platform,
            render_pass: egui_wgpu_backend::RenderPass::new(
                &gpu.device,
                wgpu::TextureFormat::Bgra8UnormSrgb,
            ),
            start_time: Instant::now(),
            previous_frame_time: None,
        }
    }

    pub fn handle_event<T>(&mut self, event: &winit::event::Event<T>) -> bool {
        self.platform.handle_event(event);
        self.platform.captures_event(event)
    }

    pub fn render(
        &mut self,
        frame: &SwapChainTexture,
        gpu: &mut Gpu,
        state: &mut State,
        physics: &mut Physics,
        render_encoder: &mut wgpu::CommandEncoder,
    ) {
        self.platform
            .update_time(self.start_time.elapsed().as_secs_f64());

        let egui_start = Instant::now();
        self.platform.begin_frame();
        let mut app_output = epi::backend::AppOutput::default();

        let mut _ui_frame = epi::backend::FrameBuilder {
            info: epi::IntegrationInfo {
                web_info: None,
                cpu_usage: self.previous_frame_time,
                seconds_since_midnight: None,
                native_pixels_per_point: Some(state.scale_factor as _),
            },
            tex_allocator: &mut self.render_pass,
            output: &mut app_output,
            repaint_signal: self.repaint_signal.clone(),
        }
        .build();

        let ctx = self.platform.context();
        ctx.set_visuals(egui::Visuals::light());

        window::ui(&ctx, state, physics, gpu);

        ctx.request_repaint();

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let (_output, paint_commands) = self.platform.end_frame();
        let paint_jobs = ctx.tessellate(paint_commands);

        let frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
        self.previous_frame_time = Some(frame_time);

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: gpu.swap_chain_desc.width,
            physical_height: gpu.swap_chain_desc.height,
            scale_factor: state.scale_factor as f32,
        };
        self.render_pass
            .update_texture(&gpu.device, &gpu.queue, &ctx.texture());
        self.render_pass
            .update_user_textures(&gpu.device, &gpu.queue);
        self.render_pass
            .update_buffers(&gpu.device, &gpu.queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        self.render_pass.execute(
            render_encoder,
            &frame.view,
            &paint_jobs,
            &screen_descriptor,
            None,
        );
    }
}
