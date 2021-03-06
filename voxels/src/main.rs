use bevy::prelude::*;

mod camera;
mod cave;
mod inspector;
mod player;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 960.0,
            height: 540.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        // .add_plugin(diagnostic::FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(inspector::InspectorPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(cave::CavePlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    // lights
    commands.insert_resource(AmbientLight {
        brightness: 0.1,
        ..default()
    });
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
        point_light: PointLight {
            intensity: 600000.,
            range: 1000.,
            shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });

    // player
    commands.spawn_bundle(player::PlayerBundle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(100.0, 0.0, 100.0),
    ));
}
