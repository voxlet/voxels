use bevy::{prelude::*, tasks::AsyncComputeTaskPool};

use crate::camera::CameraControlEvent;
use crate::player::Player;

use self::chunk::CaveChunkSettings;
use self::spawn::SpawnedCaveChunks;

mod chunk;
mod mesh;
mod pbr;
mod spawn;
mod voxelize;

pub struct CavePlugin;

impl Plugin for CavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(chunk::CaveChunkPlugin)
            .add_plugin(spawn::CaveSpawnPlugin)
            .add_plugin(voxelize::VoxelizeCaveChunkPlugin)
            .add_plugin(mesh::MeshCaveChunkPlugin)
            .add_system(pbr::insert_cave_chunk_pbr)
            .add_system(spawn_around_player)
            .add_startup_system(test_spawn);
    }
}

fn test_spawn(
    spawned_cave_chunks: Res<SpawnedCaveChunks>,
    settings: Res<CaveChunkSettings>,
    mut commands: Commands,
) {
    let task_pool = AsyncComputeTaskPool::get();

    let max_lod = 10;
    // let chunk_size: f32 = 0.64;
    // let subdivisions = 5;
    // let chunk_count = 16;
    let chunk_size: f32 = 1.28;
    let subdivisions = 5;
    let chunk_count = 8;

    let range = -chunk_count / 2..chunk_count / 2;
    let max_y = chunk_size * 2_i32.pow(max_lod) as f32;
    let min_y = -max_y;

    let inner_range = -chunk_count / 4..chunk_count / 4;
    for lod in 0..=max_lod {
        for x in range.clone() {
            for y in range.clone() {
                for z in range.clone() {
                    if lod != 0
                        && inner_range.contains(&x)
                        && inner_range.contains(&y)
                        && inner_range.contains(&z)
                    {
                        continue;
                    }

                    let size = chunk_size * 2_i32.pow(lod) as f32;
                    let origin = Vec3::new(x as f32, y as f32, z as f32) * size;
                    if origin.y < min_y || origin.y + size > max_y {
                        continue;
                    }

                    if spawned_cave_chunks.processing.len() >= task_pool.thread_num() {
                        return;
                    }

                    commands.spawn_empty().insert(spawn::spawn_cave_chunk_task(
                        task_pool,
                        chunk::CaveChunkSettings {
                            size,
                            threshold: 0.04,
                            frequency: 0.15,
                            material: settings.material.clone(),
                        },
                        origin,
                        subdivisions,
                    ));
                }
            }
        }
    }
}

fn spawn_around_player(
    mut events: EventReader<CameraControlEvent>,
    player: Query<&GlobalTransform, (With<Player>, Changed<GlobalTransform>)>,
) {
    let player = if let Ok(player) = player.get_single() {
        player
    } else {
        return;
    };

    let moved = events.iter().any(|ev| ev.delta_translation != Vec3::ZERO);
    if !moved {
        return;
    }

    let _task_pool = AsyncComputeTaskPool::get();
    info!(translation = ?player.translation());

    // let subdivisions = (settings.max_subdivisions as f32
    //     - (origin.length() / 10.0).log2().max(0.0))
    // .max(1.0) as u32;
}
