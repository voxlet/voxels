use std::time::Duration;

use rapier3d::prelude::*;
use winit::event;
// use rayon::prelude::*;

use crate::{gpu::voxel, state::State};

pub struct Physics {
    gravity: Vector<Real>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    joints: JointSet,
    ccd_solver: CCDSolver,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    query_pipeline: QueryPipeline,
}

fn voxel_world_size(voxel_resolution: usize) -> f32 {
    512.0 / voxel_resolution as f32
}

fn to_user_data(voxel: [u8; 4]) -> u128 {
    voxel[0] as u128
        + ((voxel[1] as u128) << 8)
        + ((voxel[2] as u128) << 16)
        + ((voxel[3] as u128) << 24)
}

fn new_voxel_collider_builder(voxel_world_radius: f32, user_data: u128) -> ColliderBuilder {
    ColliderBuilder::cuboid(voxel_world_radius, voxel_world_radius, voxel_world_radius)
        .friction(0.8)
        .restitution(0.3)
        .user_data(user_data)
}

fn insert_body(
    bodies: &mut RigidBodySet,
    colliders: &mut ColliderSet,
    translation: Vector<Real>,
    linvel: Vector<Real>,
    collider: Collider,
) {
    let body = RigidBodyBuilder::new_dynamic()
        .translation(translation)
        .linvel(linvel)
        .build();
    let body_handle = bodies.insert(body);
    colliders.insert_with_parent(collider, body_handle, bodies);
}

fn set_voxels(
    bodies: &mut RigidBodySet,
    colliders: &mut ColliderSet,
    voxels: &Vec<[u8; 4]>,
    size: usize,
) {
    tracing::info!("setting voxels");

    let voxel_world_size = voxel_world_size(size);
    let voxel_world_radius = voxel_world_size * 0.5;
    voxel::layers(&voxels, size)
        .iter()
        .enumerate()
        .for_each(|(z, layer)| {
            for y in 0..size {
                for x in 0..size {
                    let voxel = layer[x + y * size];
                    if voxel[3] == 0 {
                        continue;
                    };

                    let translation = vector![
                        (x as f32 + 0.5) * voxel_world_size,
                        (y as f32 + 0.5) * voxel_world_size,
                        (z as f32 + 0.5) * voxel_world_size
                    ];

                    let collider_builder =
                        new_voxel_collider_builder(voxel_world_radius, to_user_data(voxel));

                    if y == size - 1 {
                        insert_body(
                            bodies,
                            colliders,
                            translation,
                            vector!(0.0, 0.0, 0.0),
                            collider_builder.build(),
                        );
                    } else {
                        let collider = collider_builder.translation(translation).build();
                        colliders.insert(collider);
                    }
                }
            }
        });
}

impl Physics {
    pub fn new(voxels: &Vec<[u8; 4]>, size: usize) -> Self {
        /* Create other structures necessary for the simulation. */
        let gravity = vector![0.0, -9.81, 0.0];
        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let islands = IslandManager::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let joints = JointSet::new();
        let ccd_solver = CCDSolver::new();

        let mut bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();

        set_voxels(&mut bodies, &mut colliders, voxels, size);

        Self {
            gravity,
            integration_parameters,
            physics_pipeline,
            islands,
            broad_phase,
            narrow_phase,
            joints,
            ccd_solver,
            bodies,
            colliders,
            query_pipeline: QueryPipeline::new(),
        }
    }

    pub fn set_voxels(&mut self, voxels: &Vec<[u8; 4]>, size: usize) {
        *self = Self::new(voxels, size);
    }

    pub fn update(&mut self, dt: Duration) {
        self.physics_pipeline.step(
            &self.gravity,
            &IntegrationParameters {
                dt: dt.as_secs_f32().min(0.5),
                ..self.integration_parameters
            },
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &mut self.ccd_solver,
            &(),
            &(),
        )
    }

    pub fn input(&mut self, event: &event::DeviceEvent, state: &State) {
        match event {
            event::DeviceEvent::Button {
                button: 1,
                state: event::ElementState::Pressed,
            } => {
                self.query_pipeline
                    .update(&self.islands, &self.bodies, &self.colliders);

                let origin = state.camera.position * 512.0;
                let dir = state.camera.rotation * glam::vec3(0.0, 0.0, 1.0);
                tracing::info!(origin = ?origin, dir = ?dir);

                let ray = Ray::new(origin.into(), dir.into());

                if let Some((handle, _)) = self.query_pipeline.cast_ray(
                    &self.colliders,
                    &ray,
                    300.0,
                    true,
                    InteractionGroups::all(),
                    None,
                ) {
                    let colliders = &mut self.colliders;
                    if let Some(collider) = colliders.get_mut(handle) {
                        collider.user_data = 0xFFFFFFFF;
                        let linvel = (dir * 10.0).into();
                        if let Some(body_handle) = collider.parent() {
                            tracing::info!("has parent");
                            if let Some(body) = self.bodies.get_mut(body_handle) {
                                body.set_linvel(linvel, true);
                            }
                            return;
                        }
                        if let Some(collider) =
                            colliders.remove(handle, &mut self.islands, &mut self.bodies, false)
                        {
                            tracing::info!(translation = ?collider.translation(), "hit");

                            let c = new_voxel_collider_builder(
                                voxel_world_size(state.voxel_resolution) * 0.5,
                                collider.user_data,
                            );
                            insert_body(
                                &mut self.bodies,
                                colliders,
                                *collider.translation(),
                                linvel,
                                c.build(),
                            )
                        }
                    }
                } else {
                    tracing::info!("no hit");
                }
            }
            _ => {}
        }
    }

    pub fn write_voxels(&mut self, size: usize, voxels: &mut Vec<[u8; 4]>) {
        const ZERO: [u8; 4] = [0, 0, 0, 0];
        for i in 0..(size * size * size) {
            voxels[i] = ZERO;
        }

        let vws = voxel_world_size(size);

        fn to_index(pos: f32, vws: f32) -> usize {
            (pos / vws).floor() as usize
        }

        let mut out_of_bounds = Vec::new();

        self.colliders.iter().for_each(|(handle, collider)| {
            let user_data = collider.user_data;
            let t = collider.translation();
            let x = to_index(t.x, vws);
            let y = to_index(t.y, vws);
            let z = to_index(t.z, vws);

            if 0.0 <= t.x
                && x <= size - 1
                && 0.0 <= t.y
                && y <= size - 1
                && 0.0 <= t.z
                && z <= size - 1
            {
                voxels[x + y * size + z * size * size] = [
                    (user_data & 0xFF) as u8,
                    ((user_data & 0xFF00) >> 8) as u8,
                    ((user_data & 0xFF0000) >> 16) as u8,
                    ((user_data & 0xFF000000) >> 24) as u8,
                ];
            } else {
                out_of_bounds.push(handle)
            }
        });

        out_of_bounds.iter().for_each(|h| {
            self.colliders
                .remove(*h, &mut self.islands, &mut self.bodies, false);
        })
    }
}
