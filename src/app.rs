use crate::{gpu::Gpu, state, ui};

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
    ui: &mut Option<ui::Ui>,
) -> Result<(), wgpu::SwapChainError> {
    let frame = gpu.get_current_frame()?;
    let mut render_encoder = gpu.render(&frame);
    if let Some(ui) = ui {
        ui.render(&frame, gpu, state, &mut render_encoder);
    };
    gpu.queue.submit(std::iter::once(render_encoder.finish()));

    Ok(())
}

pub async fn run() {
    let event_loop: EventLoop<ui::AppEvent> = EventLoop::with_user_event();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(2560, 1440))
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(&window);
    let mut gpu = Gpu::new(&window, &state.camera).await;
    let mut ui: Option<ui::Ui> = None;
    let repaint_signal = ui::repaint_signal(&event_loop);

    event_loop.run(move |event, _, control_flow| {
        if let Some(ui) = ui.as_mut() {
            ui.handle_event(&event);
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
                        ui = Some(ui::Ui::new(&window, repaint_signal.clone(), &state, &gpu));
                    } else {
                        state::grab_cursor(&window, true);
                        ui = None;
                    }
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                update(&mut state, &mut gpu);
            }
            Event::RedrawEventsCleared => {
                match render(&mut state, &mut gpu, &mut ui) {
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => {
                        let size = state.size;
                        let scale_factor = state.scale_factor;
                        resize(&mut state, &mut gpu, size, scale_factor)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                    _ => {}
                }
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
