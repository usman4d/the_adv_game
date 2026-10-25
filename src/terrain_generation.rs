use crate::{dual_contouring::DualContouring, terrain_material::TerrainMaterial};
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    mesh::{Indices, PrimitiveTopology},
    platform::collections::HashMap,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use bevy_rapier3d::prelude::*;
pub struct TerrainGenerationPlugin;

impl Plugin for TerrainGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Terrain { ..default() })
            .insert_resource(TexturesLoading { ..default() })
            .add_plugins(MaterialPlugin::<TerrainMaterial>::default())
            .add_systems(Startup, terrain_setup)
            .add_systems(
                Update,
                (check_textures_ready, spawn_terrain_tasks, handle_terrain_tasks),
            );
    }
}
#[derive(Resource, Default)]
struct Terrain {
    chunks_map: HashMap<IVec2, Entity>,
    material_handle: Handle<StandardMaterial>,
    size_updated: bool,
}

#[derive(Component)]
struct ComputeChunk(Task<(Mesh, Collider)>);
#[derive(Component)]
pub struct TerrainSeersTag;

#[derive(Resource, Default)]
struct TexturesLoading(Vec<Handle<Image>>, bool);

fn generate_chunk(x: i32, y: i32, z: i32, size: i32) -> Mesh {
    let generator = DualContouring::new(IVec3::new(x, y, z), size);
    let vert_indicies = generator.generate();

    let indices = Indices::U32(vert_indicies.2);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for position in vert_indicies.0.iter() {
        positions.push(*position);
        uvs.push([position[0] / 17.0, position[2] / 17.0]);
    }
    for normal in vert_indicies.1.iter() {
        normals.push(*normal);
        // normals.push([0.0, 1.0, 0.0]);
    }

    //for pos in vert_indicies.0.iter() {
    //    // cube
    //    commands.spawn_bundle(PbrBundle {
    //        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
    //        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    //        transform: Transform::from_xyz(pos[0], pos[1], pos[2]),
    //        ..default()
    //    });
    //}

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_indices(indices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    // mesh.insert_attribute(Mesh::A)
    mesh.generate_tangents()
        .expect("generating tangents for terrrain chunk failed");

    mesh
}
fn terrain_setup(
    mut terrain: ResMut<Terrain>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut loading: ResMut<TexturesLoading>,
) {
    // Data textures (not colors) must be sampled without sRGB conversion.
    let linear = |settings: &mut ImageLoaderSettings| settings.is_srgb = false;
    let color_tex_handle = asset_server.load("sand_beach/vd3lecfs_2K_Albedo.jpg");
    let load_linear = |path: &'static str| {
        asset_server
            .load_builder()
            .with_settings(linear)
            .load(path)
    };
    let occlusion_texture_handle = load_linear("sand_beach/vd3lecfs_2K_AO.jpg");
    let normal_tex_handle = load_linear("sand_beach/vd3lecfs_2K_Normal.jpg");
    let roughness_tex_handle = load_linear("sand_beach/vd3lecfs_2K_Roughness.jpg");

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(color_tex_handle.clone()),
        occlusion_texture: Some(occlusion_texture_handle.clone()),
        normal_map_texture: Some(normal_tex_handle.clone()),
        metallic_roughness_texture: Some(roughness_tex_handle.clone()),
        base_color: Color::srgb(255.0 / 255.0, 211.0 / 255.0, 187.0 / 255.0),
        reflectance: 0.1,
        metallic: 0.0,
        perceptual_roughness: 1.0,
        alpha_mode: AlphaMode::Opaque,
        unlit: false,
        ..default()
    });

    loading.0.push(color_tex_handle.clone());
    loading.0.push(occlusion_texture_handle.clone());
    loading.0.push(normal_tex_handle.clone());
    loading.0.push(roughness_tex_handle.clone());

    // let material_handle = materials.add(TerrainMaterial {
    //     base_color: Color::rgb(1.0, 1.0, 1.0),
    //     color_tex_handle: color_tex_handle.clone(),
    // });

    terrain.material_handle = material_handle;
}
fn check_textures_ready(
    server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut loading: ResMut<TexturesLoading>,
) {
    if loading.1 {
        return;
    }

    if loading.0.iter().all(|h| server.is_loaded_with_dependencies(h)) {
        {
            for texture_handle in loading.0.clone().iter() {
                if let Some(mut texture) = images.get_mut(texture_handle) {
                    // do something with the texture
                    // texture.resize(Extent3d {
                    //     width: 2048,
                    //     height: 2048,
                    //     ..default()
                    // });
                    texture.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        mag_filter: ImageFilterMode::Linear,
                        min_filter: ImageFilterMode::Linear,
                        mipmap_filter: ImageFilterMode::Linear,
                        ..default()
                    });
                    loading.1 = true;
                }
            }
        }
    }
}
fn spawn_terrain_tasks(
    mut commands: Commands,
    mut terrain: ResMut<Terrain>,
    player: Query<&Transform, With<TerrainSeersTag>>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };

    let size = 32;
    let origin: Vec3 = ((player_transform.translation / size as f32).floor()) * size as f32;

    let gen_range = 8;
    let gen_range_half = gen_range / 2;

    let thread_pool = AsyncComputeTaskPool::get();

    for z_c in -gen_range_half..gen_range_half {
        for x_c in -gen_range_half..gen_range_half {
            if !terrain.chunks_map.contains_key(&IVec2::new(
                origin.x as i32 + x_c * size,
                origin.z as i32 + z_c * size,
            )) {
                let task = thread_pool.spawn(async move {
                    let chunk_mesh = generate_chunk(
                        origin.x as i32 + x_c * size,
                        -8,
                        origin.z as i32 + z_c * size,
                        size + 1,
                    );
                    let collider =
                        Collider::from_bevy_mesh(&chunk_mesh, &ComputedColliderShape::TriMesh(TriMeshFlags::default()))
                            .expect("Couldn't convert chunk mesh into a collider");

                    (chunk_mesh, collider)
                });

                //terrain plane
                let terrain_entity = commands
                    .spawn((
                        MeshMaterial3d(terrain.material_handle.clone()),
                        Transform::from_xyz(
                            origin.x + (x_c * size) as f32,
                            -8.0,
                            origin.z + (z_c * size) as f32,
                        ),
                        RigidBody::Fixed,
                        ComputeChunk(task),
                    ))
                    .id();

                terrain.chunks_map.insert(
                    IVec2::new(origin.x as i32 + x_c * size, origin.z as i32 + z_c * size),
                    terrain_entity,
                );
            }
        }
    }
}
fn handle_terrain_tasks(
    mut commands: Commands,
    mut chunk_tasks: Query<(Entity, &mut ComputeChunk)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut task) in &mut chunk_tasks {
        if let Some(chunk) = block_on(future::poll_once(&mut task.0)) {
            //terrain plane
            // commands.entity(entity).insert((
            //     mesh: meshes.add(chunk.0),
            //     ..default()
            // }).insert(chunk.1);

            commands
                .entity(entity)
                .insert((Mesh3d(meshes.add(chunk.0)), chunk.1));

            commands.entity(entity).remove::<ComputeChunk>();
        }
    }
}
