use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use block_mesh::{
    greedy_quads, ndshape::Shape, ndshape::Shape3u32, GreedyQuadsBuffer, MergeVoxel, Voxel,
    VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG,
};

pub struct CavePlugin;

impl Plugin for CavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CaveChunkSettings {
            size: 25.0,
            max_subdivisions: 7,
            threshold: 0.05,
            frequency: 0.3,
        });
        app.add_startup_system(test_spawn);
        app.add_system(voxelize_added_chunks);
    }
}

#[derive(Debug, Clone, Copy)]
struct CaveChunkSettings {
    size: f32,
    max_subdivisions: u32,
    threshold: f32,
    frequency: f32,
}

fn test_spawn(mut commands: Commands, settings: Res<CaveChunkSettings>) {
    for x in 0..50 {
        for y in 0..3 {
            for z in 0..50 {
                let origin = Vec3::new(x as f32, y as f32, z as f32);
                commands.spawn_bundle(CaveChunkBundle::new(
                    &settings,
                    Transform::from_xyz(
                        settings.size * x as f32,
                        settings.size * y as f32,
                        settings.size * z as f32,
                    ),
                    (settings.max_subdivisions as f32 - origin.length().max(1.0).log2()).max(0.0)
                        as u32,
                ));
            }
        }
    }
}

fn voxelize_added_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<CaveChunkSettings>,
    query: Query<(Entity, &CaveChunk), Added<CaveChunk>>,
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

    query.for_each(|(entity, cave_chunk)| {
        info!(entity = ?entity);

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

        let mut buffer = GreedyQuadsBuffer::new(voxels.len());
        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;
        greedy_quads(
            &voxels,
            &shape,
            [0; 3],
            [shape_length - 1; 3],
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

        let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        render_mesh.set_attribute(
            "Vertex_Position",
            VertexAttributeValues::Float32x3(positions),
        );
        render_mesh.set_attribute("Vertex_Normal", VertexAttributeValues::Float32x3(normals));
        render_mesh.set_attribute(
            "Vertex_Uv",
            VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
        );
        render_mesh.set_indices(Some(Indices::U32(indices)));

        let mesh = meshes.add(render_mesh);

        let voxel_size = settings.size / sample_count as f32;
        let index_zero_coord = -settings.size * 0.5;

        let pbr = commands
            .spawn_bundle(PbrBundle {
                mesh,
                material: material.clone(),
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

        commands.entity(entity).add_child(origin_offset);
    })
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

#[derive(Clone, Copy, Eq, PartialEq)]
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
