use super::chunk::{CaveChunkBundle, CaveChunkSettings};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

pub struct CaveSpawnPlugin;

impl Plugin for CaveSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_spawn_cave_chunk_tasks);
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct CaveChunkTask(Task<CaveChunkBundle>);

pub fn spawn_cave_chunk_task(
    task_pool: &AsyncComputeTaskPool,
    settings: CaveChunkSettings,
    origin: Vec3,
    subdivisions: u32,
) -> CaveChunkTask {
    CaveChunkTask(task_pool.spawn(async move {
        CaveChunkBundle::new(&settings, Transform::from_translation(origin), subdivisions)
    }))
}

fn handle_spawn_cave_chunk_tasks(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CaveChunkTask)>,
) {
    query.for_each_mut(|(task_entity, mut cave_chunk_task)| {
        if let Some(cave_chunk_bundle) = future::block_on(future::poll_once(&mut **cave_chunk_task))
        {
            commands.entity(task_entity).despawn();

            let subdivisions = cave_chunk_bundle.cave_chunk.subdivisions;
            let cave_chunk_entity = commands.spawn_bundle(cave_chunk_bundle);
            info!(entity = ?cave_chunk_entity.id(), subdivisions = ?subdivisions);
        }
    })
}
