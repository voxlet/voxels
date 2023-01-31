use bevy::prelude::*;

use super::{chunk::CaveChunk, mesh::CaveChunkVoxelsMeshedEvent, spawn::SpawnedCaveChunks};

pub fn insert_cave_chunk_pbr(
    mut spawned_cave_chunks: ResMut<SpawnedCaveChunks>,
    mut commands: Commands,
    mut events: EventReader<CaveChunkVoxelsMeshedEvent>,
    query: Query<&CaveChunk>,
) {
    events.iter().for_each(|ev| {
        if let Ok(cave_chunk) = query.get(ev.entity) {
            spawned_cave_chunks.processing.remove(&ev.entity);

            let mesh = if let Some(mesh) = &ev.mesh {
                mesh
            } else {
                return;
            };

            let sample_count = 2_u32.pow(cave_chunk.subdivisions);
            let voxel_size = cave_chunk.settings.size / sample_count as f32;

            let pbr = commands
                .spawn(PbrBundle {
                    mesh: mesh.clone(),
                    material: cave_chunk.settings.material.clone(),
                    transform: Transform::from_translation(Vec3::splat(-1.0)),
                    ..Default::default()
                })
                .id();

            let transform = commands
                .spawn(SpatialBundle {
                    transform: Transform::from_scale(Vec3::splat(voxel_size)),
                    ..default()
                })
                .add_child(pbr)
                .id();

            commands.entity(ev.entity).add_child(transform);

            info!(entity = ?ev.entity)
        }
    });
}
