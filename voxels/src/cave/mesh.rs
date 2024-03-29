use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
    tasks::{AsyncComputeTaskPool, Task},
};
use block_mesh::{greedy_quads, ndshape::Shape, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG};
use futures_lite::future;

use super::voxelize::{CaveChunkVoxelizedEvent, CaveChunkVoxels};

pub struct MeshCaveChunkPlugin;

impl Plugin for MeshCaveChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mesh_cave_chunk_voxels);
        app.add_event::<CaveChunkVoxelsMeshedEvent>();
        app.add_systems(Update, handle_mesh_cave_chunk_voxels_tasks);
    }
}

fn mesh_cave_chunk_voxels(
    mut commands: Commands,
    mut events: EventReader<CaveChunkVoxelizedEvent>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    events.iter().for_each(|ev| {
        commands
            .spawn_empty()
            .insert(spawn_mesh_cave_chunk_voxels_task(
                task_pool,
                ev.entity,
                ev.voxels.clone(),
            ));
    })
}

#[derive(Component, Deref, DerefMut)]
// struct MeshCaveChunkVoxelsTask(Task<Option<(Entity, CaveChunkVoxels, Mesh)>>);
struct MeshCaveChunkVoxelsTask(Task<Option<(Entity, Option<Mesh>)>>);

fn spawn_mesh_cave_chunk_voxels_task(
    task_pool: &AsyncComputeTaskPool,
    cave_chunk_entity: Entity,
    cave_chunk_voxels: CaveChunkVoxels,
) -> MeshCaveChunkVoxelsTask {
    MeshCaveChunkVoxelsTask(task_pool.spawn(async move {
        let locked = cave_chunk_voxels.data.try_read().ok()?;
        let voxels = if let Some(voxels) = &*locked {
            voxels
        } else {
            return Some((
                cave_chunk_entity,
                // cave_chunk_voxels.clone(),
                None,
            ));
        };

        let mut buffer = GreedyQuadsBuffer::new(voxels.len());
        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;
        greedy_quads(
            voxels,
            &cave_chunk_voxels.shape,
            [0; 3],
            [cave_chunk_voxels.shape.as_array()[0] - 1; 3],
            &faces,
            &mut buffer,
        );

        let num_indices = buffer.quads.num_quads() * 6;
        let num_vertices = buffer.quads.num_quads() * 4;
        let mut indices = Vec::with_capacity(num_indices);
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);
        for (group, face) in buffer.quads.groups.iter().zip(faces.iter()) {
            for quad in group.iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                positions.extend_from_slice(&face.quad_mesh_positions(quad, 1.0));
                normals.extend_from_slice(&face.quad_mesh_normals());
            }
        }

        let mut cave_chunk_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        cave_chunk_mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        );
        cave_chunk_mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3(normals),
        );
        cave_chunk_mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
        );
        cave_chunk_mesh.set_indices(Some(Indices::U32(indices)));

        Some((
            cave_chunk_entity,
            // cave_chunk_voxels.clone(),
            Some(cave_chunk_mesh),
        ))
    }))
}

#[derive(Event)]
pub struct CaveChunkVoxelsMeshedEvent {
    pub entity: Entity,
    // pub voxels: CaveChunkVoxels,
    pub mesh: Option<Handle<Mesh>>,
}

fn handle_mesh_cave_chunk_voxels_tasks(
    mut commands: Commands,
    mut events: EventWriter<CaveChunkVoxelsMeshedEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut MeshCaveChunkVoxelsTask)>,
) {
    query.for_each_mut(|(task_entity, mut cave_chunk_task)| {
        if let Some(result) = future::block_on(future::poll_once(&mut **cave_chunk_task)) {
            commands.entity(task_entity).despawn();

            // if let Some((entity, _voxels, mesh)) = result {
            if let Some((entity, mesh)) = result {
                events.send(CaveChunkVoxelsMeshedEvent {
                    entity,
                    // voxels,
                    mesh: mesh.map(|m| meshes.add(m)),
                });
            }
        }
    })
}
