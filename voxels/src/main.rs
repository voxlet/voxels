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
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world
    let mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.45,
        subdivisions: 32,
    }));

    for y in -2..=2 {
        for x in -5..=5 {
            let x01 = (x + 5) as f32 / 10.0;
            let y01 = (y + 2) as f32 / 4.0;
            // sphere
            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: materials.add(StandardMaterial {
                    base_color: Color::hex("ffd891").unwrap(),
                    // vary key PBR parameters on a grid of spheres to show the effect
                    metallic: y01,
                    perceptual_roughness: x01,
                    ..Default::default()
                }),
                transform: Transform::from_xyz(x as f32, y as f32 + 0.5, 0.0),
                ..Default::default()
            });
        }
    }
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(50.0, 50.0, 50.0)),
        point_light: PointLight {
            intensity: 600000.,
            range: 100.,
            ..Default::default()
        },
        ..Default::default()
    });
}
