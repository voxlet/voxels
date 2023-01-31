use std::time::Instant;

use crate::{
    gpu::{voxel, Gpu},
    physics::Physics,
    state, ui,
};

use winit::{
    dpi::PhysicalSize,
    event::{self, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{self, WindowBuilder},
};

use crate::state::State;

fn resize(state: &mut State, gpu: &mut Gpu, size: PhysicalSize<u32>, scale_factor: f64) {
    state.resize(size, scale_factor);
    gpu.resize(size.width, size.height);
}

fn input(
    window: &window::Window,
    event: &event::DeviceEvent,
    state: &mut State,
    physics: &mut Physics,
) {
    state.input(window, event);
    physics.input(event, state);
}

fn update(state: &mut State, physics: &mut Physics, gpu: &mut Gpu) {
    let dt = state.update();
    let now = Instant::now();
    physics.update(dt);
    state.physics_step_time = now.elapsed();
    let now = Instant::now();
    physics.write_voxels(state.voxel_resolution, &mut state.voxels);
    state.physics_write_time = now.elapsed();
    let now = Instant::now();
    gpu.update_voxels(&state.voxels, state.voxel_resolution);
    state.voxel_transfer_time = now.elapsed();
    gpu.update(dt, &state.camera);
}

fn render(
    state: &mut State,
    physics: &mut Physics,
    gpu: &mut Gpu,
    ui: &mut Option<ui::Ui>,
) -> Result<(), wgpu::SurfaceError> {
    let frame = gpu.surface.get_current_texture()?;
    let mut render_encoder = gpu.render(&frame);
    if let Some(ui) = ui {
        ui.render(&frame, gpu, state, physics, &mut render_encoder);
    };
    gpu.queue.submit(std::iter::once(render_encoder.finish()));

    Ok(())
}

pub fn set_voxel_resolution(
    state: &mut State,
    physics: &mut Physics,
    gpu: &mut Gpu,
    voxel_resolution: usize,
) {
    state.voxel_resolution = voxel_resolution;
    state.voxels = voxel::caves(voxel_resolution);
    physics.set_voxels(&state.voxels, voxel_resolution);
    gpu.set_voxels(&state.voxels, voxel_resolution);
}

pub async fn run() {
    let event_loop: EventLoop<ui::AppEvent> = EventLoop::with_user_event();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::<u32>::new(1920, 1080))
        .with_always_on_top(true)
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(&window);
    let mut physics = Physics::new(&state.voxels, state.voxel_resolution);
    let mut gpu = Gpu::new(
        &window,
        &state.voxels,
        state.voxel_resolution,
        &state.camera,
    )
    .await;
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
                update(&mut state, &mut physics, &mut gpu);
            }
            Event::RedrawEventsCleared => {
                match render(&mut state, &mut physics, &mut gpu, &mut ui) {
                    // Recreate the swap_chain if lost
                    Err(wgpu::SurfaceError::Lost) => {
                        let size = state.size;
                        let scale_factor = state.scale_factor;
                        resize(&mut state, &mut gpu, size, scale_factor)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
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
                input(&window, event, &mut state, &mut physics);
            }
            _ => {}
        }
    });
}
