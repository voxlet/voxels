use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore},
    input::common_conditions::input_toggle_active,
    prelude::*,
};
use bevy_egui::{egui, EguiContexts};
use bevy_inspector_egui::{
    egui::Ui,
    quick::{ResourceInspectorPlugin, WorldInspectorPlugin},
};
use std::time::Instant;

use crate::camera::CameraControlSettings;

#[derive(Reflect, Resource, Default, Debug)]
#[reflect(Resource)]
struct Inspector {
    camera_control_settings: CameraControlSettings,
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum InspectorState {
    Active,
    #[default]
    Inactive,
}

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_egui::EguiPlugin)
            .add_plugins(
                WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
            )
            .add_plugins(ResourceInspectorPlugin::<Inspector>::new())
            .register_type::<Inspector>()
            // .add_systems(Update, toggle)
            .add_systems(Update, ui)
            // .add_system_set(SystemSet::on_enter(InspectorState::Active).with_system(activate))
            // .add_system_set(SystemSet::on_enter(InspectorState::Inactive).with_system(deactivate))
            .add_state::<InspectorState>();
    }
}

// fn toggle(key: Res<Input<KeyCode>>, mut state: ResMut<State<InspectorState>>) {
//     if key.just_pressed(KeyCode::Escape) {
//         let next_state = match state.get() {
//             InspectorState::Active => InspectorState::Inactive,
//             _ => InspectorState::Active,
//         };
//         state.set(next_state).ok();
//     }
// }

fn ui(
    mut egui_ctx: EguiContexts,
    time: Res<Time>,
    state: Res<State<InspectorState>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    if *state.get() == InspectorState::Inactive {
        return;
    }
    egui::Window::new("Metrics")
        .resizable(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            diagnostics_ui(ui, diagnostics, time.startup())
        });
}

fn diagnostics_ui(ui: &mut Ui, diagnostics: Res<DiagnosticsStore>, start_time: Instant) {
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
                plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from_iter(
                    diagnostic
                        .measurements()
                        .map(|m| [m.time.duration_since(start_time).as_secs_f64(), m.value]),
                )))
            });
    }
    ui.end_row();
}

// fn activate(world_inspector: ResMut<WorldInspectorParams>, inspectors: ResMut<InspectorWindows>) {
//     update_active(true, world_inspector, inspectors);
// }

// fn deactivate(world_inspector: ResMut<WorldInspectorParams>, inspectors: ResMut<InspectorWindows>) {
//     update_active(false, world_inspector, inspectors);
// }

// fn update_active(
//     active: bool,
//     mut world_inspector: ResMut<WorldInspectorParams>,
//     mut inspectors: ResMut<InspectorWindows>,
// ) {
//     world_inspector.enabled = active;
//     inspectors.window_data_mut::<Inspector>().visible = active;
// }
