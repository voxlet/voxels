use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use smooth_bevy_cameras::controllers::fps::ControlEvent;
use smooth_bevy_cameras::LookTransform;

use crate::player::Player;

use super::chunk::{CaveChunkBundle, CaveChunkSettings};

pub struct CaveSpawnPlugin;

impl Plugin for CaveSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_spawn_cave_chunk_tasks)
            .add_system(spawn_around_player);
    }
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

fn spawn_around_player(
    _task_pool: Res<AsyncComputeTaskPool>,
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

    info!(translation = ?player.translation);
}
