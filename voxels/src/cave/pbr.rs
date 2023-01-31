use bevy::prelude::*;

use super::{
    chunk::{CaveChunk, CaveChunkSettings},
    mesh::CaveChunkVoxelsMeshedEvent,
};

pub fn insert_cave_chunk_pbr(
    mut commands: Commands,
    settings: Res<CaveChunkSettings>,
    mut events: EventReader<CaveChunkVoxelsMeshedEvent>,
    query: Query<&CaveChunk>,
) {
    events.iter().for_each(|ev| {
        if let Some(cave_chunk) = query.get(ev.entity).ok() {
            let sample_count = 2_u32.pow(cave_chunk.subdivisions);
            let voxel_size = settings.size / sample_count as f32;
            let index_zero_coord = -settings.size * 0.5;

            let pbr = commands
                .spawn_bundle(PbrBundle {
                    mesh: ev.mesh.clone(),
                    material: settings.material.clone(),
                    transform: Transform::from_translation(Vec3::splat(-1.0)),
                    ..Default::default()
                })
                .id();

            let origin_offset = commands
                .spawn()
                .insert(
                    Transform::from_translation(Vec3::splat(index_zero_coord))
                        .with_scale(Vec3::splat(voxel_size)),
                )
                .insert(GlobalTransform::default())
                .add_child(pbr)
                .id();

            commands.entity(ev.entity).add_child(origin_offset);
            info!(entity = ?ev.entity)
        }
    });
}
