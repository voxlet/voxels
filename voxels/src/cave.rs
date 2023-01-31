use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
    tasks::{AsyncComputeTaskPool, Task},
};
use block_mesh::{
    greedy_quads, ndshape::Shape, ndshape::Shape3u32, GreedyQuadsBuffer, MergeVoxel, Voxel,
    VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG,
};
use futures_lite::future;

pub struct CavePlugin;

impl Plugin for CavePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ReadyEvent>();
        app.add_startup_system(insert_settings);
        app.add_system(test_spawn);
        app.add_system(voxelize_cave_chunks);
        app.add_system(mesh_cave_chunk_voxels);
        app.add_system(insert_cave_chunk_pbr);
    }
}

struct ReadyEvent(CaveChunkSettings);

fn insert_settings(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ready_event: EventWriter<ReadyEvent>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::hex("ffd891").unwrap(),
        metallic: 0.5,
        perceptual_roughness: 0.5,
        ..Default::default()
    });

    // let edge_material = materials.add(StandardMaterial {
    //     base_color: Color::hex("ffff22").unwrap(),
    //     metallic: 1.0,
    //     perceptual_roughness: 0.1,
    //     ..Default::default()
    // });

    let settings = CaveChunkSettings {
        size: 6.4,
        max_subdivisions: 8,
        threshold: 0.05,
        frequency: 0.15,
        material,
    };
    commands.insert_resource(settings.clone());

    ready_event.send(ReadyEvent(settings));
}

#[derive(Debug, Clone)]
struct CaveChunkSettings {
    size: f32,
    max_subdivisions: u32,
    threshold: f32,
    frequency: f32,
    material: Handle<StandardMaterial>,
}

fn test_spawn(
    mut commands: Commands,
    mut ready_event: EventReader<ReadyEvent>,
    task_pool: Res<AsyncComputeTaskPool>,
) {
    ready_event.iter().for_each(|ready_event| {
        let settings = &ready_event.0;
        for x in 0..32 {
            for y in -4..5 {
                for z in 0..32 {
                    let origin = Vec3::new(x as f32, y as f32, z as f32) * settings.size;
                    let subdivisions = (settings.max_subdivisions as f32
                        - (origin.length() / 20.0).log2().max(0.0))
                    .max(1.0) as u32;

                    commands.spawn().insert(spawn_cave_chunk_task(
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

type CaveChunkTask = Task<CaveChunkBundle>;

fn spawn_cave_chunk_task(
    task_pool: &AsyncComputeTaskPool,
    settings: CaveChunkSettings,
    origin: Vec3,
    subdivisions: u32,
) -> CaveChunkTask {
    task_pool.spawn(async move {
        CaveChunkBundle::new(&settings, Transform::from_translation(origin), subdivisions)
    })
}

fn voxelize_cave_chunks(
    mut commands: Commands,
    task_pool: Res<AsyncComputeTaskPool>,
    settings: Res<CaveChunkSettings>,
    mut query: Query<(Entity, &mut CaveChunkTask)>,
) {
    query.for_each_mut(|(entity, mut cave_chunk_task)| {
        if let Some(cave_chunk_bundle) = future::block_on(future::poll_once(&mut *cave_chunk_task))
        {
            commands.entity(entity).despawn();

            commands.spawn().insert(spawn_voxelize_cave_chunk_task(
                &task_pool,
                settings.clone(),
                cave_chunk_bundle,
            ));
        }
    })
}

type VoxelizeCaveChunkTask = Task<(CaveChunkBundle, CaveChunkVoxels)>;

fn spawn_voxelize_cave_chunk_task(
    task_pool: &AsyncComputeTaskPool,
    settings: CaveChunkSettings,
    cave_chunk_bundle: CaveChunkBundle,
) -> VoxelizeCaveChunkTask {
    task_pool.spawn(async move {
        let cave_chunk = &cave_chunk_bundle.cave_chunk;

        let sample_count = 2_u32.pow(cave_chunk.subdivisions);
        let shape_length = sample_count + 2;
        let shape = Shape3u32::new([shape_length, shape_length, shape_length]);

        let mut voxels: Vec<BoolVoxel> = Vec::with_capacity(shape.size() as usize);

        let y_stride = sample_count;
        let z_stride = sample_count * y_stride;

        for i in 0..shape.size() {
            let [x, y, z] = shape.delinearize(i);
            if x == 0
                || y == 0
                || z == 0
                || x == shape_length - 1
                || y == shape_length - 1
                || z == shape_length - 1
            {
                voxels.push(BoolVoxel(false));
            } else {
                let noise_index = (x - 1 + (y - 1) * y_stride + (z - 1) * z_stride) as usize;
                voxels.push(BoolVoxel(
                    cave_chunk.noise_samples[noise_index as usize] > settings.threshold,
                ))
            }
        }

        (cave_chunk_bundle, CaveChunkVoxels { voxels, shape })
    })
}

fn mesh_cave_chunk_voxels(
    mut commands: Commands,
    task_pool: Res<AsyncComputeTaskPool>,
    mut query: Query<(Entity, &mut VoxelizeCaveChunkTask)>,
) {
    query.for_each_mut(|(entity, mut voxelize_cave_chunk_task)| {
        if let Some((cave_chunk_bundle, cave_chunk_voxels)) =
            future::block_on(future::poll_once(&mut *voxelize_cave_chunk_task))
        {
            commands.entity(entity).despawn();

            commands.spawn().insert(spawn_mesh_cave_chunk_voxels_task(
                &task_pool,
                cave_chunk_bundle,
                cave_chunk_voxels,
            ));
        }
    })
}

type MeshCaveChunkVoxelsTask = Task<(CaveChunkBundle, CaveChunkVoxels, Mesh)>;

fn spawn_mesh_cave_chunk_voxels_task(
    task_pool: &AsyncComputeTaskPool,
    cave_chunk_bundle: CaveChunkBundle,
    cave_chunk_voxels: CaveChunkVoxels,
) -> MeshCaveChunkVoxelsTask {
    task_pool.spawn(async move {
        let mut buffer = GreedyQuadsBuffer::new(cave_chunk_voxels.voxels.len());
        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;
        greedy_quads(
            &cave_chunk_voxels.voxels,
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
                positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
                normals.extend_from_slice(&face.quad_mesh_normals());
            }
        }

        let mut cave_chunk_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        cave_chunk_mesh.set_attribute(
            "Vertex_Position",
            VertexAttributeValues::Float32x3(positions),
        );
        cave_chunk_mesh.set_attribute("Vertex_Normal", VertexAttributeValues::Float32x3(normals));
        cave_chunk_mesh.set_attribute(
            "Vertex_Uv",
            VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
        );
        cave_chunk_mesh.set_indices(Some(Indices::U32(indices)));

        (cave_chunk_bundle, cave_chunk_voxels, cave_chunk_mesh)
    })
}

fn insert_cave_chunk_pbr(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    settings: Res<CaveChunkSettings>,
    mut query: Query<(Entity, &mut MeshCaveChunkVoxelsTask)>,
) {
    query.for_each_mut(|(entity, mut mesh_cave_chunk_voxels_task)| {
        if let Some((cave_chunk_bundle, _cave_chunk_voxels, cave_chunk_mesh)) =
            future::block_on(future::poll_once(&mut *mesh_cave_chunk_voxels_task))
        {
            commands.entity(entity).despawn();

            let mesh = meshes.add(cave_chunk_mesh);

            let sample_count = 2_u32.pow(cave_chunk_bundle.cave_chunk.subdivisions);
            let voxel_size = settings.size / sample_count as f32;
            let index_zero_coord = -settings.size * 0.5;

            let pbr = commands
                .spawn_bundle(PbrBundle {
                    mesh,
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

            info!(origin = ?cave_chunk_bundle.transform.translation, subdivisions = ?cave_chunk_bundle.cave_chunk.subdivisions);

            commands
                .spawn_bundle(cave_chunk_bundle)
                .add_child(origin_offset);
        }
    });
}

#[derive(Bundle, Default)]
struct CaveChunkBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    cave_chunk: CaveChunk,
}

impl CaveChunkBundle {
    fn new(settings: &CaveChunkSettings, transform: Transform, subdivisions: u32) -> Self {
        CaveChunkBundle {
            transform,
            cave_chunk: CaveChunk::new(settings, transform.translation, subdivisions),
            ..Default::default()
        }
    }
}

#[derive(Component, Default, Debug)]
struct CaveChunk {
    subdivisions: u32,
    noise_samples: Vec<f32>,
}

impl CaveChunk {
    fn new(settings: &CaveChunkSettings, origin: Vec3, subdivisions: u32) -> Self {
        let sample_count = 2_usize.pow(subdivisions);
        let half_voxel_size = settings.size * 0.5 / sample_count as f32;

        let (noise_samples, min, max) = simdnoise::NoiseBuilder::fbm_3d_offset(
            origin.x * sample_count as f32 / settings.size + half_voxel_size,
            sample_count,
            origin.y * sample_count as f32 / settings.size + half_voxel_size,
            sample_count,
            origin.z * sample_count as f32 / settings.size + half_voxel_size,
            sample_count,
        )
        .with_seed(42)
        .with_freq(settings.frequency * settings.size / sample_count as f32)
        .generate();

        info!(origin = ?origin, subdivisions = ?subdivisions, min = ?min, max = ?max);
        CaveChunk {
            subdivisions,
            noise_samples,
        }
    }
}

#[derive(Component)]
struct CaveChunkVoxels {
    voxels: Vec<BoolVoxel>,
    shape: Shape3u32,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
struct BoolVoxel(bool);

const EMPTY: BoolVoxel = BoolVoxel(false);

impl Voxel for BoolVoxel {
    #[inline]
    fn get_visibility(&self) -> VoxelVisibility {
        if *self == EMPTY {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for BoolVoxel {
    type MergeValue = Self;

    #[inline]
    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}
