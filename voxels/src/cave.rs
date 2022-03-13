use bevy::prelude::*;

pub struct CavePlugin;

impl Plugin for CavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CaveChunkSettings {
            size: 10.0,
            subdivisions: 3,
            threshold: 0.7,
            frequency: 0.25,
        });
        app.add_startup_system(test_spawn);
        app.add_system(voxelize);
    }
}

#[derive(Debug, Clone, Copy)]
struct CaveChunkSettings {
    size: f32,
    subdivisions: u32,
    threshold: f32,
    frequency: f32,
}

fn test_spawn(mut commands: Commands, settings: Res<CaveChunkSettings>) {
    for x in 0..10 {
        for y in 0..5 {
            for z in 0..10 {
                commands.spawn_bundle(CaveChunkBundle::new(
                    Transform::from_xyz(
                        settings.size * x as f32,
                        settings.size * y as f32,
                        settings.size * z as f32,
                    ),
                    &settings,
                ));
            }
        }
    }
}

fn voxelize(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<CaveChunkSettings>,
    query: Query<(Entity, &CaveChunk), Added<CaveChunk>>,
) {
    let sample_count = 2_u32.pow(settings.subdivisions);

    let voxel_size = settings.size / sample_count as f32;
    let index_zero_coord = voxel_size * 0.5 - settings.size * 0.5;

    let mesh = meshes.add(Mesh::from(shape::Cube { size: voxel_size }));
    let material = materials.add(StandardMaterial {
        base_color: Color::hex("ffd891").unwrap(),
        metallic: 0.5,
        perceptual_roughness: 0.5,
        ..Default::default()
    });
    let edge_material = materials.add(StandardMaterial {
        base_color: Color::hex("ffff22").unwrap(),
        metallic: 1.0,
        perceptual_roughness: 0.1,
        ..Default::default()
    });
    for (entity, cave_chunk) in query.iter() {
        info!(entity = ?entity);
        let y_stride = sample_count;
        let z_stride = sample_count * y_stride;
        for x_index in 0..sample_count {
            for y_index in 0..sample_count {
                for z_index in 0..sample_count {
                    let noise_index = (x_index + y_index * y_stride + z_index * z_stride) as usize;
                    if cave_chunk.noise_samples[noise_index] < settings.threshold {
                        continue;
                    }
                    let voxel = commands
                        .spawn_bundle(PbrBundle {
                            mesh: mesh.clone(),
                            material: if x_index == 0 || y_index == 0 || z_index == 0 {
                                edge_material.clone()
                            } else {
                                material.clone()
                            },
                            transform: Transform::from_xyz(
                                index_zero_coord + voxel_size * x_index as f32,
                                index_zero_coord + voxel_size * y_index as f32,
                                index_zero_coord + voxel_size * z_index as f32,
                            ),
                            ..Default::default()
                        })
                        .id();
                    commands.entity(entity).add_child(voxel);
                }
            }
        }
    }
}

#[derive(Bundle, Default)]
struct CaveChunkBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    cave_chunk: CaveChunk,
}

impl CaveChunkBundle {
    fn new(transform: Transform, settings: &CaveChunkSettings) -> Self {
        CaveChunkBundle {
            transform,
            cave_chunk: CaveChunk::new(transform.translation, settings),
            ..Default::default()
        }
    }
}

#[derive(Component, Default, Debug)]
struct CaveChunk {
    noise_samples: Vec<f32>,
}

impl CaveChunk {
    fn new(origin: Vec3, settings: &CaveChunkSettings) -> Self {
        info!(origin = ?origin);
        let sample_count = 2_usize.pow(settings.subdivisions);

        let noise_samples = simdnoise::NoiseBuilder::fbm_3d_offset(
            origin.x * sample_count as f32 / settings.size,
            sample_count,
            origin.y * sample_count as f32 / settings.size,
            sample_count,
            origin.z * sample_count as f32 / settings.size,
            sample_count,
        )
        .with_seed(42)
        .with_freq(settings.frequency * settings.size / sample_count as f32)
        .generate_scaled(0.0, 1.0);

        CaveChunk { noise_samples }
    }
}
