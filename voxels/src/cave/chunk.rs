use std::sync::{Arc, RwLock};

use bevy::prelude::*;

pub struct CaveChunkPlugin;

impl Plugin for CaveChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ReadyEvent>();
        app.add_startup_system(insert_settings);
    }
}

pub struct ReadyEvent {
    pub settings: CaveChunkSettings,
}

pub fn insert_settings(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ready_event: EventWriter<ReadyEvent>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::hex("ffd891").unwrap(),
        metallic: 0.5,
        perceptual_roughness: 0.5,
        ..Default::default()
    });

    // let edge_material = materials.add(StandardMaterial {
    //     base_color: Color::hex("ffff22").unwrap(),
    //     metallic: 1.0,
    //     perceptual_roughness: 0.1,
    //     ..Default::default()
    // });

    let settings = CaveChunkSettings {
        size: 655.36,
        max_subdivisions: 5,
        threshold: 0.04,
        frequency: 0.15,
        material,
    };
    commands.insert_resource(settings.clone());

    ready_event.send(ReadyEvent { settings });
}

#[derive(Debug, Clone)]
pub struct CaveChunkSettings {
    pub size: f32,
    pub max_subdivisions: u32,
    pub threshold: f32,
    pub frequency: f32,
    pub material: Handle<StandardMaterial>,
}

#[derive(Bundle, Default)]
pub struct CaveChunkBundle {
    #[bundle]
    pub spatial: SpatialBundle,
    pub cave_chunk: CaveChunk,
}

impl CaveChunkBundle {
    pub fn new(settings: &CaveChunkSettings, transform: Transform, subdivisions: u32) -> Self {
        CaveChunkBundle {
            spatial: SpatialBundle {
                transform,
                ..default()
            },
            cave_chunk: CaveChunk::new(settings, transform.translation, subdivisions),
        }
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct CaveChunk {
    pub subdivisions: u32,
    pub noise_samples: Arc<RwLock<Vec<f32>>>,
}

impl CaveChunk {
    fn new(settings: &CaveChunkSettings, origin: Vec3, subdivisions: u32) -> Self {
        let sample_count = 2_usize.pow(subdivisions);
        let half_voxel_size = settings.size * 0.5 / sample_count as f32;

        let (noise_samples, _min, _max) = simdnoise::NoiseBuilder::fbm_3d_offset(
            origin.x * sample_count as f32 / settings.size + half_voxel_size,
            sample_count,
            origin.y * sample_count as f32 / settings.size + half_voxel_size,
            sample_count,
            origin.z * sample_count as f32 / settings.size + half_voxel_size,
            sample_count,
        )
        .with_seed(42)
        .with_freq(settings.frequency * settings.size / sample_count as f32)
        .generate();

        CaveChunk {
            subdivisions,
            noise_samples: Arc::new(RwLock::new(noise_samples)),
        }
    }
}
