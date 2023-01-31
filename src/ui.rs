mod window;

use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
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

impl epi::backend::RepaintSignal for RepaintSignal {
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
                1,
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
        frame: &wgpu::SurfaceTexture,
        gpu: &mut Gpu,
        state: &mut State,
        physics: &mut Physics,
        render_encoder: &mut wgpu::CommandEncoder,
    ) {
        self.platform
            .update_time(self.start_time.elapsed().as_secs_f64());

        let egui_start = Instant::now();
        self.platform.begin_frame();
        let app_output = epi::backend::AppOutput::default();

        let mut _ui_frame = epi::Frame::new(epi::backend::FrameData {
            info: epi::IntegrationInfo {
                name: "debug_console",
                web_info: None,
                cpu_usage: self.previous_frame_time,
                native_pixels_per_point: Some(state.scale_factor as _),
                prefer_dark_mode: None,
            },
            output: app_output,
            repaint_signal: self.repaint_signal.clone(),
        });

        let ctx = self.platform.context();
        ctx.set_visuals(egui::Visuals::light());

        window::ui(&ctx, state, physics, gpu);

        ctx.request_repaint();

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let (_output, paint_commands) = self.platform.end_frame(None);
        let paint_jobs = ctx.tessellate(paint_commands);

        let frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
        self.previous_frame_time = Some(frame_time);

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: gpu.surface_config.width,
            physical_height: gpu.surface_config.height,
            scale_factor: state.scale_factor as f32,
        };
        self.render_pass
            .update_texture(&gpu.device, &gpu.queue, &ctx.font_image());
        self.render_pass
            .update_user_textures(&gpu.device, &gpu.queue);
        self.render_pass
            .update_buffers(&gpu.device, &gpu.queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Surface View (Debug Console)"),
            ..Default::default()
        });
        self.render_pass
            .execute(render_encoder, &view, &paint_jobs, &screen_descriptor, None).unwrap();
    }
}
