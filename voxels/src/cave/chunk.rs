use std::sync::{Arc, RwLock};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;

pub struct CaveChunkPlugin;

impl Plugin for CaveChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ReadyEvent>();
        app.add_startup_system(insert_settings);
        app.add_system(handle_spawn_cave_chunk_tasks);
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
        size: 12.8,
        max_subdivisions: 9,
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

type CaveChunkTask = Task<CaveChunkBundle>;

pub fn spawn_cave_chunk_task(
    task_pool: &AsyncComputeTaskPool,
    settings: CaveChunkSettings,
    origin: Vec3,
    subdivisions: u32,
) -> CaveChunkTask {
    task_pool.spawn(async move {
        CaveChunkBundle::new(&settings, Transform::from_translation(origin), subdivisions)
    })
}

fn handle_spawn_cave_chunk_tasks(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CaveChunkTask)>,
) {
    query.for_each_mut(|(task_entity, mut cave_chunk_task)| {
        if let Some(cave_chunk_bundle) = future::block_on(future::poll_once(&mut *cave_chunk_task))
        {
            commands.entity(task_entity).despawn();

            let subdivisions = cave_chunk_bundle.cave_chunk.subdivisions;
            let cave_chunk_entity = commands.spawn_bundle(cave_chunk_bundle).id();
            info!(entity = ?cave_chunk_entity, subdivisions = ?subdivisions);
        }
    })
}

#[derive(Bundle, Default)]
pub struct CaveChunkBundle {
    pub transform: Transform,
    global_transform: GlobalTransform,
    pub cave_chunk: CaveChunk,
}

impl CaveChunkBundle {
    fn new(settings: &CaveChunkSettings, transform: Transform, subdivisions: u32) -> Self {
        CaveChunkBundle {
            transform,
            cave_chunk: CaveChunk::new(settings, transform.translation, subdivisions),
            ..Default::default()
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
