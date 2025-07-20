use crate::CHUNK_WIDTH;
use crate::atlas_material::{ATTRIBUTE_TEXTURE_INDEX, AtlasMaterial};
use crate::block::{Block, Oclusion};
use crate::chunk_blocks::ChunkBlocks;
use crate::chunks::{Chunk, ChunksIndex, Loader, local_to_global};
use crate::spacial::{QUAD_INDICES, QUAD_UV, Side, Sides, SidesExt, Sign};
use crate::water_material::WaterMaterial;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology::TriangleList};

#[derive(Component)]
pub struct NeedsRemeshing;

#[derive(Component, Clone, Copy)]
pub struct HasMesh {
    regular: Entity,
    water: Entity,
}

#[derive(Resource)]
pub struct MeshAssets {
    atlas_material: Handle<AtlasMaterial>,
    water_material: Handle<WaterMaterial>,
}

type Candidate = (
    With<ChunkBlocks>,
    Or<(Without<HasMesh>, With<NeedsRemeshing>)>,
);

pub fn chunks_mesh_setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlas_material: ResMut<Assets<AtlasMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
) {
    commands.insert_resource(MeshAssets {
        atlas_material: atlas_material.add(AtlasMaterial::new(
            "assets/textures/blocks.png",
            16,
            images.as_mut(),
        )),
        water_material: water_materials.add(WaterMaterial::new(
            "assets/textures/water.png",
            images.as_mut(),
        )),
    });
}

pub fn chunk_meshing(
    index: Res<ChunksIndex>,
    blocks: Query<&ChunkBlocks>,
    chunks: Query<(Entity, &Chunk, Option<&HasMesh>), Candidate>,
    loaders: Query<(&Transform, &Loader)>,
    assets: Res<MeshAssets>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, &chunk, has_mesh) in &chunks {
        if loaders.iter().any(|(transform, &loader)| {
            loader.inside_zone(transform.translation, chunk, Loader::ZONE_MESH)
        }) {
            let has_blocks = |n| index.get(n).map(|e| blocks.contains(e)).unwrap_or(false);
            if chunk.neighbours().list().all(has_blocks) {
                let has_mesh = if let Some(&has_mesh) = has_mesh {
                    has_mesh
                } else {
                    let regular = commands
                        .spawn((
                            Name::new("ChunkMeshRegular"),
                            Transform::default(),
                            ChildOf(entity),
                        ))
                        .id();
                    let water = commands
                        .spawn((
                            Name::new("ChunkMeshWater"),
                            Transform::default(),
                            ChildOf(entity),
                        ))
                        .id();
                    commands.entity(entity).insert(HasMesh { regular, water });
                    HasMesh { regular, water }
                };
                let (regular, water) = chunk_build_mesh(&index, blocks, chunk);
                commands.entity(entity).remove::<NeedsRemeshing>();
                commands.entity(has_mesh.regular).insert((
                    Mesh3d(meshes.add(regular)),
                    MeshMaterial3d(assets.atlas_material.clone()),
                ));
                commands.entity(has_mesh.water).insert((
                    Mesh3d(meshes.add(water)),
                    MeshMaterial3d(assets.water_material.clone()),
                ));
            }
        }
    }
}

pub fn chunk_demeshing(
    chunks: Query<(Entity, &Chunk, Option<&HasMesh>), Or<(With<HasMesh>, With<NeedsRemeshing>)>>,
    loaders: Query<(&Transform, &Loader)>,
    mut commands: Commands,
) {
    for (entity, &chunk, has_mesh) in chunks {
        if loaders.iter().all(|(transform, &loader)| {
            loader.outside_zone(transform.translation, chunk, Loader::ZONE_MESH)
        }) {
            commands
                .entity(entity)
                .remove::<(NeedsRemeshing, HasMesh)>();
            if let Some(has_mesh) = has_mesh {
                commands.entity(has_mesh.regular).despawn();
                commands.entity(has_mesh.water).despawn();
            }
        }
    }
}

pub fn chunk_build_mesh(
    index: &ChunksIndex,
    blocks: Query<&ChunkBlocks>,
    chunk: Chunk,
) -> (Mesh, Mesh) {
    let mut regular_positions: Vec<[f32; 3]> = Vec::new();
    let mut regular_normals: Vec<[f32; 3]> = Vec::new();
    let mut regular_texture_uvs = Vec::new();
    let mut regular_texture_indices = Vec::new();
    let mut regular_indices = Vec::new();

    let mut water_positions: Vec<[f32; 3]> = Vec::new();
    let mut water_normals: Vec<[f32; 3]> = Vec::new();
    let mut water_texture_uvs = Vec::new();
    let mut water_texture_indices = Vec::new();
    let mut water_indices = Vec::new();
    // let entity = index.get(chunk).unwrap();
    // let chunk_blocks = blocks.get(entity).unwrap();

    for x in 0..CHUNK_WIDTH {
        for y in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                let local = IVec3 { x, y, z };
                let global = local_to_global(chunk, local);
                let block = index.get_block(|e| blocks.get(e), global).unwrap();

                make_cube_mesh2(
                    local.as_vec3(),
                    block,
                    Sides::NORMAL.map(|v: IVec3| {
                        index
                            .get_block(|e| blocks.get(e), global + v)
                            .unwrap_or_else(|| {
                                warn!(
                                    "trying to fetch block from non-generated chunk during meshing"
                                );
                                Block::Air
                            })
                    }),
                    |quad| match quad {
                        Quad::Regular {
                            positions,
                            normal,
                            texture_uv,
                            texture_index,
                        } => {
                            let index_shift = regular_positions.len();
                            for position in positions {
                                regular_positions.push(position.into());
                                regular_normals.push(normal.into());
                            }
                            for texture_uv in texture_uv {
                                regular_texture_uvs.push(texture_uv);
                                regular_texture_indices.push(texture_index);
                            }
                            for index in QUAD_INDICES {
                                regular_indices.push(index + index_shift as u32);
                            }
                        }
                        Quad::Water {
                            positions,
                            normal,
                            texture_uv,
                        } => {
                            let index_shift = water_positions.len();
                            for position in positions {
                                water_positions.push(position.into());
                                water_normals.push(normal.into());
                            }
                            for texture_uv in texture_uv {
                                water_texture_uvs.push(texture_uv);
                                water_texture_indices.push(0u32);
                            }
                            for index in QUAD_INDICES {
                                water_indices.push(index + index_shift as u32);
                            }
                        }
                    },
                );
            }
        }
    }
    (
        Mesh::new(TriangleList, default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, regular_positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, regular_normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, regular_texture_uvs)
            .with_inserted_attribute(ATTRIBUTE_TEXTURE_INDEX, regular_texture_indices)
            .with_inserted_indices(Indices::U32(regular_indices)),
        Mesh::new(TriangleList, default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, water_positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, water_normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, water_texture_uvs)
            .with_inserted_attribute(ATTRIBUTE_TEXTURE_INDEX, water_texture_indices)
            .with_inserted_indices(Indices::U32(water_indices)),
    )
}

enum Quad {
    Regular {
        positions: [Vec3; 4],
        normal: Vec3,
        texture_uv: [[f32; 2]; 4],
        texture_index: u32,
    },
    Water {
        positions: [Vec3; 4],
        normal: Vec3,
        texture_uv: [[f32; 2]; 4],
    },
}

fn make_cube_mesh2(position: Vec3, block: Block, sides: Sides<Block>, mut write: impl FnMut(Quad)) {
    if block == Block::Water {
        for side in Side::ALL {
            if sides[side].oclusion() == Oclusion::Full || sides[side] == Block::Water {
                continue;
            }
            write(Quad::Water {
                positions: side.quad().map(|v: Vec3| position + v),
                normal: side.normal(),
                texture_uv: QUAD_UV,
            });
        }
    } else {
        let Some(textures) = block.textures() else {
            return;
        };
        for side in Side::ALL {
            if sides[side].oclusion() == Oclusion::Full {
                continue;
            }
            let ([uv_swap, u_flip, v_flip], texture_index) = textures[side];
            let texture_uv = QUAD_UV.map(|[u, v]| {
                let [u, v] = match uv_swap {
                    Sign::Pos => [u, v],
                    Sign::Neg => [v, u],
                };
                let u = match u_flip {
                    Sign::Pos => 0.0 + u,
                    Sign::Neg => 1.0 - u,
                };
                let v = match v_flip {
                    Sign::Pos => 0.0 + v,
                    Sign::Neg => 1.0 - v,
                };
                [u, v]
            });
            write(Quad::Regular {
                positions: side.quad().map(|v: Vec3| position + v),
                normal: side.normal(),
                texture_uv,
                texture_index,
            });
        }
    }
}
