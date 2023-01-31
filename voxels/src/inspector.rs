use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorParams;
use smooth_bevy_cameras::controllers::fps::FpsCameraController;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new())
            .add_startup_system(setup)
            .add_system(toggle_inspector);
    }
}

fn setup(mut world_inspector: ResMut<WorldInspectorParams>) {
    world_inspector.enabled = false;
}

fn toggle_inspector(
    input: Res<Input<KeyCode>>,
    mut world_inspector: ResMut<WorldInspectorParams>,
    mut cameras: Query<&mut FpsCameraController>,
) {
    if input.just_pressed(KeyCode::Escape) {
        let show_inspector = !world_inspector.enabled;
        world_inspector.enabled = show_inspector;
        for mut c in cameras.iter_mut() {
            c.enabled = !show_inspector
        }
    }
}
