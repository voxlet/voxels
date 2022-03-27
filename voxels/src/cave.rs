use bevy::{prelude::*, tasks::AsyncComputeTaskPool};

mod chunk;
mod mesh;
mod pbr;
mod voxelize;

pub struct CavePlugin;

impl Plugin for CavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(chunk::CaveChunkPlugin);
        app.add_plugin(voxelize::VoxelizeCaveChunkPlugin);
        app.add_plugin(mesh::MeshCaveChunkPlugin);
        app.add_system(pbr::insert_cave_chunk_pbr);
        app.add_system(test_spawn);
    }
}

fn test_spawn(
    mut commands: Commands,
    mut ready_event: EventReader<chunk::ReadyEvent>,
    task_pool: Res<AsyncComputeTaskPool>,
) {
    ready_event.iter().for_each(|ready_event| {
        let settings = &ready_event.settings;
        for x in -1..15 {
            for y in -2..3 {
                for z in -1..15 {
                    let origin = Vec3::new(x as f32, y as f32, z as f32) * settings.size;
                    let subdivisions = (settings.max_subdivisions as f32
                        - (origin.length() / 10.0).log2().max(0.0))
                    .max(1.0) as u32;

                    commands.spawn().insert(chunk::spawn_cave_chunk_task(
                        &task_pool,
                        settings.clone(),
                        origin,
                        subdivisions,
                    ));
                }
            }
        }
    });
}
