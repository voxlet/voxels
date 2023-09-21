use bevy::{diagnostic, prelude::*};

mod camera;
mod cave;
mod inspector;
mod player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (960.0, 540.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(diagnostic::FrameTimeDiagnosticsPlugin)
        // .add_plugin(diagnostic::LogDiagnosticsPlugin::default())
        // .add_plugin(WireframePlugin)
        .add_plugins((
            inspector::InspectorPlugin,
            camera::CameraPlugin,
            cave::CavePlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // lights
    commands.insert_resource(AmbientLight {
        brightness: 0.1,
        ..default()
    });
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(10.0, 30.0, 10.0)),
        point_light: PointLight {
            intensity: 200000.,
            range: 1000.,
            // shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });

    // player
    commands.spawn(player::PlayerBundle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -10.0),
    ));
}
