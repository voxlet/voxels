use egui::{CtxRef, Ui};

use crate::state::State;

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

pub fn ui(ctx: &CtxRef, state: &mut State) {
    egui::Window::new("Debug").show(ctx, |ui| {
        camera(ui, state);
    });
}
