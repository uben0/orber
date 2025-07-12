use crate::atlas_material::{ATTRIBUTE_TEXTURE_INDEX, AtlasMaterial};
use crate::chunk_blocks::ChunkBlocks;
use crate::chunks::{Chunk, ChunksIndex, Loader, assert_is_local, local_to_global};
use crate::spacial::{Side, Sides, SidesExt};
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
            if chunk.neighbours().all(has_blocks) {
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
    let entity = index.get(chunk).unwrap();
    for (&local, ()) in &blocks.get(entity).unwrap().blocks {
        let global = local_to_global(chunk, local);
        make_cube_mesh(
            local,
            &mut positions,
            &mut normals,
            &mut texture_uvs,
            &mut texture_indices,
            &mut indices,
            Sides::NORMAL.map(|v: IVec3| {
                !index
                    .get_block(|e| blocks.get(e), global + v)
                    .unwrap_or(false)
            }),
        );
    }
    Mesh::new(TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, texture_uvs)
        .with_inserted_attribute(ATTRIBUTE_TEXTURE_INDEX, texture_indices)
        .with_inserted_indices(Indices::U32(indices))
}

fn make_cube_mesh(
    local: IVec3,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    texture_uvs: &mut Vec<[f32; 2]>,
    texture_indices: &mut Vec<u32>,
    indices: &mut Vec<u32>,
    visible: Sides<bool>,
) {
    assert_is_local(local);
    for side in Side::ALL {
        if visible[side] {
            let index = positions.len() as u32;
            positions.extend(side.quad().map(|v| <[f32; 3]>::from(v + local.as_vec3())));
            normals.extend([side.normal(); 4]);
            texture_uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
            texture_indices.extend([0; 4]);
            indices.extend([
                index + 0,
                index + 1,
                index + 2,
                index + 2,
                index + 3,
                index + 0,
            ]);
        }
    }
}
