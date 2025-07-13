use crate::CHUNK_WIDTH;
use crate::atlas_material::{ATTRIBUTE_TEXTURE_INDEX, AtlasMaterial};
use crate::block::{Block, Oclusion};
use crate::chunk_blocks::ChunkBlocks;
use crate::chunks::{Chunk, ChunksIndex, Loader, local_to_global};
use crate::spacial::{QUAD_INDICES, QUAD_UV, Side, Sides, SidesExt, Sign, Vec3Ext};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology::TriangleList};

#[derive(Component)]
pub struct NeedsRemeshing;

#[derive(Resource)]
pub struct MeshAssets {
    material: Handle<AtlasMaterial>,
}

type Candidate = (
    With<ChunkBlocks>,
    Or<(Without<Mesh3d>, With<NeedsRemeshing>)>,
);

pub fn chunks_mesh_setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<AtlasMaterial>>,
) {
    commands.insert_resource(MeshAssets {
        material: materials.add(AtlasMaterial::new(
            "assets/textures/blocks.png",
            16,
            images.as_mut(),
        )),
    });
}

pub fn chunk_meshing(
    index: Res<ChunksIndex>,
    blocks: Query<&ChunkBlocks>,
    chunks: Query<(Entity, &Chunk), Candidate>,
    loaders: Query<(&Transform, &Loader)>,
    assets: Res<MeshAssets>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, &chunk) in &chunks {
        if loaders.iter().any(|(transform, &loader)| {
            loader.inside_zone(transform.translation, chunk, Loader::ZONE_MESH)
        }) {
            let has_blocks = |n| index.get(n).map(|e| blocks.contains(e)).unwrap_or(false);
            if chunk.neighbours().list().all(has_blocks) {
                let mesh = chunk_build_mesh(&index, blocks, chunk);
                commands.entity(entity).remove::<NeedsRemeshing>().insert((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(assets.material.clone()),
                ));
            }
        }
    }
}

pub fn chunk_demeshing(
    chunks: Query<(Entity, &Chunk), Or<(With<Mesh3d>, With<NeedsRemeshing>)>>,
    loaders: Query<(&Transform, &Loader)>,
    mut commands: Commands,
) {
    for (entity, &chunk) in chunks {
        if loaders.iter().all(|(transform, &loader)| {
            loader.outside_zone(transform.translation, chunk, Loader::ZONE_MESH)
        }) {
            commands
                .entity(entity)
                .remove::<(NeedsRemeshing, Mesh3d, MeshMaterial3d<AtlasMaterial>)>();
        }
    }
}

pub fn chunk_build_mesh(index: &ChunksIndex, blocks: Query<&ChunkBlocks>, chunk: Chunk) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut texture_uvs = Vec::new();
    let mut texture_indices = Vec::new();
    let mut indices = Vec::new();
    // let entity = index.get(chunk).unwrap();
    // let chunk_blocks = blocks.get(entity).unwrap();

    for x in 0..CHUNK_WIDTH {
        for y in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                let local = IVec3 { x, y, z };
                let global = local_to_global(chunk, local);

                let block = index.get_block(|e| blocks.get(e), global).unwrap();

                if let Some(textures) = block.textures() {
                    make_cube_mesh(
                        local.as_vec3(),
                        &mut positions,
                        &mut normals,
                        &mut texture_uvs,
                        &mut texture_indices,
                        &mut indices,
                        textures,
                        Sides::NORMAL.map(|v: IVec3| {
                            index
                                .get_block(|e| blocks.get(e), global + v)
                                .unwrap_or(Block::Air)
                                .oclusion()
                                != Oclusion::Full
                        }),
                    );
                }
            }
        }
    }
    Mesh::new(TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, texture_uvs)
        .with_inserted_attribute(ATTRIBUTE_TEXTURE_INDEX, texture_indices)
        .with_inserted_indices(Indices::U32(indices))
}

fn make_cube_mesh(
    parent: Vec3,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    texture_uvs: &mut Vec<[f32; 2]>,
    texture_indices: &mut Vec<u32>,
    indices: &mut Vec<u32>,
    textures: Sides<([Sign; 3], u32)>,
    visible: Sides<bool>,
) {
    for side in Side::ALL {
        if visible[side] {
            let index = positions.len() as u32;
            positions.extend(side.quad().map(|v| v.zips(parent, |l, r| l + r)));
            normals.extend([side.normal(); 4]);
            let flip_logit = |logit: f32, sign| match sign {
                Sign::Pos => 0.0 + logit,
                Sign::Neg => 1.0 - logit,
            };
            let swap_pair = |[lhs, rhs]: [f32; 2], sign| match sign {
                Sign::Pos => [lhs, rhs],
                Sign::Neg => [rhs, lhs],
            };
            let ([uv_swap, u_flip, v_flip], texture_index) = textures[side];
            texture_uvs.extend(
                QUAD_UV
                    .map(|uv| swap_pair(uv, uv_swap))
                    .map(|[u, v]| [flip_logit(u, u_flip), flip_logit(v, v_flip)]),
            );
            texture_indices.extend([texture_index; 4]);
            indices.extend(QUAD_INDICES.map(|i| i + index));
        }
    }
}
