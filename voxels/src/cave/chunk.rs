use std::sync::{Arc, RwLock};

use bevy::prelude::*;

pub struct CaveChunkPlugin;

impl Plugin for CaveChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ReadyEvent>();
        app.add_startup_system(insert_settings);
    }
}

#[derive(Clone, Debug)]
pub struct ReadyEvent {
    pub settings: CaveChunkSettings,
}

pub fn insert_settings(world: &mut World) {
    let mut materials = world
        .get_resource_mut::<Assets<StandardMaterial>>()
        .unwrap();
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
        size: 0.64,
        threshold: 0.04,
        frequency: 0.15,
        material,
    };
    world.insert_resource(settings);

    // ready_event.send(ReadyEvent { settings });
}

#[derive(Resource, Debug, Clone)]
pub struct CaveChunkSettings {
    pub size: f32,
    pub threshold: f32,
    pub frequency: f32,
    pub material: Handle<StandardMaterial>,
}

#[derive(Bundle)]
pub struct CaveChunkBundle {
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

#[derive(Component, Debug, Clone)]
pub struct CaveChunk {
    pub subdivisions: u32,
    pub noise_samples: Arc<RwLock<Vec<f32>>>,
    pub settings: CaveChunkSettings,
}

#[derive(Component, Debug)]
pub struct CaveChunkOriginTask {
    pub task_entity: Entity,
}

impl CaveChunk {
    fn new(settings: &CaveChunkSettings, origin: Vec3, subdivisions: u32) -> Self {
        let sample_count = 2_usize.pow(subdivisions);
        let voxel_size = settings.size / sample_count as f32;
        info!(
            subdivisions = subdivisions,
            sample_count = sample_count,
            voxel_size = voxel_size
        );

        let (noise_samples, _min, _max) = simdnoise::NoiseBuilder::fbm_3d_offset(
            origin.x / voxel_size,
            sample_count,
            origin.y / voxel_size,
            sample_count,
            origin.z / voxel_size,
            sample_count,
        )
        .with_seed(42)
        .with_freq(settings.frequency * voxel_size)
        .generate();

        CaveChunk {
            subdivisions,
            noise_samples: Arc::new(RwLock::new(noise_samples)),
            settings: settings.clone(),
        }
    }
}
