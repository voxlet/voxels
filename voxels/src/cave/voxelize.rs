use std::sync::{Arc, RwLock};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use block_mesh::{
    ndshape::{RuntimeShape, Shape},
    MergeVoxel, Voxel, VoxelVisibility,
};
use futures_lite::future;

use super::chunk::CaveChunk;

pub struct VoxelizeCaveChunkPlugin;

impl Plugin for VoxelizeCaveChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CaveChunkNeedsVoxelizingEvent>();
        app.add_systems(Update, detect_added_cave_chunks);
        app.add_systems(Update, voxelize_cave_chunks);
        app.add_event::<CaveChunkVoxelizedEvent>();
        app.add_systems(Update, handle_voxelize_cave_chunk_tasks);
    }
}

#[derive(Event)]
struct CaveChunkNeedsVoxelizingEvent {
    entity: Entity,
}

fn detect_added_cave_chunks(
    mut events: EventWriter<CaveChunkNeedsVoxelizingEvent>,
    query: Query<(Entity, &CaveChunk), Added<CaveChunk>>,
) {
    query.for_each(|(entity, _)| events.send(CaveChunkNeedsVoxelizingEvent { entity }))
}

fn voxelize_cave_chunks(
    mut commands: Commands,
    mut events: EventReader<CaveChunkNeedsVoxelizingEvent>,
    query: Query<&CaveChunk>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    events.iter().for_each(|ev| {
        if let Ok(cave_chunk) = query.get(ev.entity) {
            commands
                .spawn_empty()
                .insert(spawn_voxelize_cave_chunk_task(
                    task_pool,
                    ev.entity,
                    cave_chunk.clone(),
                ));
        }
    })
}

#[derive(Component, Deref, DerefMut)]
struct VoxelizeCaveChunkTask(Task<Option<CaveChunkVoxelizedEvent>>);

fn spawn_voxelize_cave_chunk_task(
    task_pool: &AsyncComputeTaskPool,
    cave_chunk_entity: Entity,
    cave_chunk: CaveChunk,
) -> VoxelizeCaveChunkTask {
    VoxelizeCaveChunkTask(task_pool.spawn(async move {
        let noise_samples = cave_chunk.noise_samples.try_read().ok()?;

        let sample_count = 2_u32.pow(cave_chunk.subdivisions);
        let shape_length = sample_count + 2;
        let shape = RuntimeShape::<u32, 3>::new([shape_length, shape_length, shape_length]);

        let mut voxels: Vec<BoolVoxel> = Vec::with_capacity(shape.size() as usize);

        let y_stride = sample_count;
        let z_stride = sample_count * y_stride;

        let mut empty = true;
        for i in 0..shape.size() {
            let [x, y, z] = shape.delinearize(i);
            if x == 0
                || y == 0
                || z == 0
                || x == shape_length - 1
                || y == shape_length - 1
                || z == shape_length - 1
            {
                voxels.push(BoolVoxel(false));
            } else {
                let noise_index = (x - 1 + (y - 1) * y_stride + (z - 1) * z_stride) as usize;
                let value = noise_samples[noise_index] > cave_chunk.settings.threshold;
                empty = empty && !value;
                voxels.push(BoolVoxel(value))
            }
        }


        let data = if empty { None } else { Some(voxels) };
        info!(entity = ?cave_chunk_entity, size = cave_chunk.settings.size, subdivisions = ?cave_chunk.subdivisions);

        Some(CaveChunkVoxelizedEvent {
            entity: cave_chunk_entity,
            voxels: CaveChunkVoxels {
                data: Arc::new(RwLock::new(data)),
                shape,
            },
        })
    }))
}

#[derive(Event)]
pub struct CaveChunkVoxelizedEvent {
    pub entity: Entity,
    pub voxels: CaveChunkVoxels,
}

fn handle_voxelize_cave_chunk_tasks(
    mut commands: Commands,
    mut events: EventWriter<CaveChunkVoxelizedEvent>,
    mut query: Query<(Entity, &mut VoxelizeCaveChunkTask)>,
) {
    query.for_each_mut(|(entity, mut cave_chunk_task)| {
        if let Some(result) = future::block_on(future::poll_once(&mut **cave_chunk_task)) {
            commands.entity(entity).despawn();

            if let Some(event) = result {
                events.send(event);
            }
        }
    })
}

#[derive(Component, Clone)]
pub struct CaveChunkVoxels {
    pub data: Arc<RwLock<Option<Vec<BoolVoxel>>>>,
    pub shape: RuntimeShape<u32, 3>,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
pub struct BoolVoxel(bool);

const EMPTY: BoolVoxel = BoolVoxel(false);

impl Voxel for BoolVoxel {
    #[inline]
    fn get_visibility(&self) -> VoxelVisibility {
        if *self == EMPTY {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for BoolVoxel {
    type MergeValue = Self;

    #[inline]
    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}
