use std::time::Duration;

use rapier3d::prelude::*;
// use rayon::prelude::*;

use crate::gpu::voxel;

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
}

fn voxel_world_size(voxel_resolution: usize) -> f32 {
    512.0 / voxel_resolution as f32
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

                    let collider_builder = ColliderBuilder::cuboid(
                        voxel_world_radius,
                        voxel_world_radius,
                        voxel_world_radius,
                    )
                    .restitution(0.5)
                    .user_data(
                        voxel[0] as u128
                            + ((voxel[1] as u128) << 8)
                            + ((voxel[2] as u128) << 16)
                            + ((voxel[3] as u128) << 24),
                    );

                    if y > (size as f32 * 0.9) as usize {
                        let body = RigidBodyBuilder::new_dynamic()
                            .translation(translation)
                            .linvel(vector![0.0, 0.01, 0.0])
                            .build();
                        let body_handle = bodies.insert(body);
                        colliders.insert_with_parent(collider_builder.build(), body_handle, bodies);
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
        }
    }

    pub fn set_voxels(&mut self, voxels: &Vec<[u8; 4]>, size: usize) {
        *self = Self::new(voxels, size);
    }

    pub fn update(&mut self, _dt: Duration) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
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

    pub fn write_voxels(&self, size: usize, voxels: &mut Vec<[u8; 4]>) {
        const ZERO: [u8; 4] = [0, 0, 0, 0];
        for i in 0..(size * size * size) {
            voxels[i] = ZERO;
        }

        let vws = voxel_world_size(size);

        fn to_index(pos: f32, vws: f32, size: usize) -> usize {
            ((pos / vws).floor() as usize).clamp(0, size - 1)
        }

        self.colliders.iter().for_each(|(_, collider)| {
            let user_data = collider.user_data;
            let t = collider.translation();
            let x = to_index(t.x, vws, size);
            let y = to_index(t.y, vws, size);
            let z = to_index(t.z, vws, size);

            voxels[x + y * size + z * size * size] = [
                (user_data & 0xFF) as u8,
                ((user_data & 0xFF00) >> 8) as u8,
                ((user_data & 0xFF0000) >> 16) as u8,
                ((user_data & 0xFF000000) >> 24) as u8,
            ]
        })
    }
}
