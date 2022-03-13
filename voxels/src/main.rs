use bevy::{diagnostic, prelude::*};

mod camera;
mod cave;
mod inspector;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugin(diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(inspector::InspectorPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(cave::CavePlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
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
