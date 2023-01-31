use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use smooth_bevy_cameras::controllers::fps::ControlEvent;
use smooth_bevy_cameras::LookTransform;

use crate::player::Player;

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
            .add_system(test_spawn);
    }
}

fn test_spawn(mut commands: Commands, mut ready_event: EventReader<chunk::ReadyEvent>) {
    ready_event.iter().for_each(|ready_event| {
        let settings = &ready_event.settings;
        let task_pool = AsyncComputeTaskPool::get();

        let chunk_size: f32 = 0.64;
        let subdivisions = 5;
        let max_lod = 4;
        let max_y = chunk_size * 2_i32.pow(max_lod) as f32;
        let min_y = -max_y;

        let inner_range = -4..4;
        for level in 0..=max_lod {
            for x in -8..8 {
                for y in -8..8 {
                    for z in -8..8 {
                        if level != 0
                            && inner_range.contains(&x)
                            && inner_range.contains(&y)
                            && inner_range.contains(&z)
                        {
                            continue;
                        }

                        let size = chunk_size * 2_i32.pow(level) as f32;
                        let origin = Vec3::new(x as f32, y as f32, z as f32) * size;
                        if origin.y < min_y || origin.y + size > max_y {
                            continue;
                        }

                        commands.spawn().insert(spawn::spawn_cave_chunk_task(
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
    });
}

fn spawn_around_player(
    mut events: EventReader<ControlEvent>,
    player: Query<&GlobalTransform, (With<Player>, Changed<LookTransform>)>,
) {
    let player = if let Ok(player) = player.get_single() {
        player
    } else {
        return;
    };

    let mut moved = false;
    for ev in events.iter() {
        // add more as needed
        #[allow(clippy::single_match)]
        match ev {
            &ControlEvent::TranslateEye(_) => {
                moved = true;
                break;
            }
            _ => {}
        }
    }
    if !moved {
        return;
    }

    let _task_pool = AsyncComputeTaskPool::get();
    info!(translation = ?player.translation());

    // let subdivisions = (settings.max_subdivisions as f32
    //     - (origin.length() / 10.0).log2().max(0.0))
    // .max(1.0) as u32;
}
