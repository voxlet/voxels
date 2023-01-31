use crate::{
    gpu::Gpu,
    state,
    ui::{self, AppEvent, Ui},
};

use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::state::State;

pub fn resize(state: &mut State, gpu: &mut Gpu, size: PhysicalSize<u32>, scale_factor: f64) {
    state.resize(size, scale_factor);
    gpu.resize(size.width, size.height);
}

pub fn update(state: &mut State, gpu: &mut Gpu) {
    let dt = state.update();
    gpu.update(dt, &state.camera);
}

pub fn render(
    state: &mut State,
    gpu: &mut Gpu,
    ui: &mut Option<Ui>,
    control_flow: &mut ControlFlow,
) {
    match gpu.get_current_frame() {
        // Recreate the swap_chain if lost
        Err(wgpu::SwapChainError::Lost) => resize(state, gpu, state.size, state.scale_factor),
        // The system is out of memory, we should probably quit
        Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
        // All other errors (Outdated, Timeout) should be resolved by the next frame
        Err(e) => eprintln!("{:?}", e),
        Ok(frame) => {
            gpu.render(&frame);
            if let Some(ui) = ui {
                ui.render(&frame, gpu, state);
            }
        }
    }
}

pub async fn run() {
    let event_loop: EventLoop<AppEvent> = EventLoop::with_user_event();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1920, 1080))
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(&window);
    let mut gpu = Gpu::new(&window, &state.camera).await;
    let mut ui: Option<Ui> = None;
    let repaint_signal = ui::repaint_signal(&event_loop);

    event_loop.run(move |event, _, control_flow| {
        if let Some(ui) = ui.as_mut() {
            ui.platform.handle_event(&event);
            if ui.platform.captures_event(&event) {
                return;
            }
        }

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    let scale_factor = state.scale_factor;
                    resize(&mut state, &mut gpu, *physical_size, scale_factor);
                }
                WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    scale_factor,
                } => {
                    resize(&mut state, &mut gpu, **new_inner_size, *scale_factor);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    if let None = ui {
                        state::grab_cursor(&window, false);
                        ui = Some(Ui::new(&window, repaint_signal.clone(), &state, &gpu))
                    } else {
                        state::grab_cursor(&window, true);
                        ui = None;
                    }
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                update(&mut state, &mut gpu);
                render(&mut state, &mut gpu, &mut ui, control_flow);
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            Event::DeviceEvent { ref event, .. } => {
                state.input(&window, event);
            }
            _ => {}
        }
    });
}
