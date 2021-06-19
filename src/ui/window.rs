use egui::{ComboBox, CtxRef, Ui};

use crate::{app, gpu::Gpu, physics::Physics, state::State};

pub fn camera(ui: &mut Ui, state: &mut State) {
    let camera = &mut state.camera;
    ui.horizontal(|ui| {
        let position = &mut camera.position;
        ui.label("Position");
        ui.label(format!("{:.2}", position.x));
        ui.label(format!("{:.2}", position.y));
        ui.label(format!("{:.2}", position.z));
    });
    ui.horizontal(|ui| {
        ui.label("Yaw");
        ui.label(format!("{:.2}", state.camera.yaw));
        ui.label("Pitch");
        ui.label(format!("{:.2}", state.camera.pitch));
    });
}

pub fn frame_time(ui: &mut Ui, state: &mut State) {
    ui.horizontal(|ui| {
        ui.label("Frame Time");
        ui.label(format!("{:.2}", state.dt.as_micros() as f32 / 1000.0));
    });
    ui.horizontal(|ui| {
        ui.label("Physics Step Time");
        ui.label(format!(
            "{:.2}",
            state.physics_step_time.as_micros() as f32 / 1000.0
        ));
    });
    ui.horizontal(|ui| {
        ui.label("Physics Write Time");
        ui.label(format!(
            "{:.2}",
            state.physics_write_time.as_micros() as f32 / 1000.0
        ));
    });
    ui.horizontal(|ui| {
        ui.label("Voxel Transfer Time");
        ui.label(format!(
            "{:.2}",
            state.voxel_transfer_time.as_micros() as f32 / 1000.0
        ));
    });
}

pub fn voxel_resolution(ui: &mut Ui, state: &mut State, physics: &mut Physics, gpu: &mut Gpu) {
    let voxel_res = state.voxel_resolution;
    ComboBox::from_label("Voxel Resolution")
        .selected_text(voxel_res as u32)
        .show_ui(ui, |ui| {
            let mut voxel_res = state.voxel_resolution;
            ui.selectable_value(&mut voxel_res, 64, "64");
            ui.selectable_value(&mut voxel_res, 128, "128");
            ui.selectable_value(&mut voxel_res, 256, "256");
            ui.selectable_value(&mut voxel_res, 512, "512");
            if voxel_res != state.voxel_resolution {
                app::set_voxel_resolution(state, physics, gpu, voxel_res)
            }
        });
}

pub fn ui(ctx: &CtxRef, state: &mut State, physics: &mut Physics, gpu: &mut Gpu) {
    egui::Window::new("Debug").show(ctx, |ui| {
        frame_time(ui, state);
        camera(ui, state);
        voxel_resolution(ui, state, physics, gpu);
    });
}
