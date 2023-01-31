use std::time::Instant;

use bevy::{
    diagnostic::{Diagnostic, Diagnostics},
    prelude::*,
};
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::{egui::Ui, WorldInspectorParams};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum InspectorState {
    Active,
    Inactive,
}

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_egui::EguiPlugin)
            .add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new())
            .add_system(toggle)
            .add_system(ui)
            .add_system_set(SystemSet::on_enter(InspectorState::Active).with_system(activate))
            .add_system_set(SystemSet::on_enter(InspectorState::Inactive).with_system(deactivate))
            .add_state(InspectorState::Inactive);
    }
}

fn toggle(key: Res<Input<KeyCode>>, mut state: ResMut<State<InspectorState>>) {
    if key.just_pressed(KeyCode::Escape) {
        let next_state = match state.current() {
            InspectorState::Active => InspectorState::Inactive,
            _ => InspectorState::Active,
        };
        state.set(next_state).ok();
    }
}

fn ui(
    time: Res<Time>,
    state: Res<State<InspectorState>>,
    mut egui_ctx: ResMut<EguiContext>,
    diagnostics: Res<Diagnostics>,
) {
    if *state.current() == InspectorState::Inactive {
        return;
    }
    egui::Window::new("Inspector")
        .resizable(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            diagnostics_ui(ui, diagnostics, time.startup())
        });
}

fn diagnostics_ui(ui: &mut Ui, diagnostics: Res<Diagnostics>, start_time: Instant) {
    //egui::Grid::new("Diagnostics").show(ui, |ui| {
    for diagnostic in diagnostics.iter() {
        diagnostic_ui(ui, diagnostic, start_time);
    }
    //});
}

fn diagnostic_ui(ui: &mut Ui, diagnostic: &Diagnostic, start_time: Instant) {
    let name = diagnostic.name.as_ref();
    ui.horizontal(|ui| {
        ui.label(name);
        if let Some(value) = diagnostic.value() {
            ui.label(format!("{:.5}", value));
            if let Some(avg) = diagnostic.average() {
                if avg != value {
                    ui.label(format!("avg: {:.5}", avg));
                }
            }
        }
    });
    if diagnostic.history_len() > 1 {
        egui::plot::Plot::new(name)
            .view_aspect(3.0)
            .include_y(0.0)
            .show(ui, |plot_ui| {
                plot_ui.line(egui::plot::Line::new(egui::plot::Values::from_values_iter(
                    diagnostic.measurements().map(|m| egui::plot::Value {
                        x: m.time.duration_since(start_time).as_secs_f64(),
                        y: m.value,
                    }),
                )))
            });
    }
    ui.end_row();
}

fn activate(world_inspector: ResMut<WorldInspectorParams>) {
    update_active(true, world_inspector);
}

fn deactivate(world_inspector: ResMut<WorldInspectorParams>) {
    update_active(false, world_inspector);
}

fn update_active(active: bool, mut world_inspector: ResMut<WorldInspectorParams>) {
    world_inspector.enabled = active;
}
