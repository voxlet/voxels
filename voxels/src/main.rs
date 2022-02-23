use std::time::Instant;

use bevy::{diagnostic, prelude::*};

mod camera;
mod inspector;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugin(diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(inspector::InspectorPlugin)
        .add_plugin(camera::CameraPlugin)
        .init_resource::<GlobalState>()
        .add_startup_system(setup)
        .run();
}

#[derive(Debug)]
struct GlobalState {
    start_time: Instant,
}

impl FromWorld for GlobalState {
    fn from_world(_: &mut World) -> Self {
        GlobalState {
            start_time: Instant::now(),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const WIDTH: usize = 100;
    const HEIGHT: usize = 100;
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let material = materials.add(StandardMaterial {
        base_color: Color::PINK,
        ..Default::default()
    });
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            // cube
            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_xyz((x as f32) * 2.0, (y as f32) * 2.0, 0.0),
                ..Default::default()
            });
        }
    }
}
