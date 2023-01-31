use egui::{ComboBox, CtxRef, Ui};

use crate::{gpu::Gpu, state::State};

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
}

pub fn voxel_size(ui: &mut Ui, gpu: &mut Gpu) {
    let voxel_size = gpu.state.data.voxel_size;
    ComboBox::from_label("Voxel Resolution")
        .selected_text((1.0 / voxel_size) as u32)
        .show_ui(ui, |ui| {
            let mut voxel_size = gpu.state.data.voxel_size;
            ui.selectable_value(&mut voxel_size, 1.0 / 64.0, "64");
            ui.selectable_value(&mut voxel_size, 1.0 / 128.0, "128");
            ui.selectable_value(&mut voxel_size, 1.0 / 256.0, "256");
            ui.selectable_value(&mut voxel_size, 1.0 / 512.0, "512");
            if voxel_size != gpu.state.data.voxel_size {
                gpu.set_voxel_size(voxel_size);
            }
        });
}

pub fn ui(ctx: &CtxRef, state: &mut State, gpu: &mut Gpu) {
    egui::Window::new("Debug").show(ctx, |ui| {
        frame_time(ui, state);
        camera(ui, state);
        voxel_size(ui, gpu);
    });
}
