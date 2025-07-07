use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::mesh::{Mesh, PrimitiveTopology::TriangleList};

use crate::chunks::{Chunk, ChunksIndex, local_to_global};
use crate::spacial::Sides;
use crate::{blocks::ChunkBlocks, make_cube_mesh};

pub fn chunk_meshing(
    index: Res<ChunksIndex>,
    blocks: Query<&ChunkBlocks>,
    chunks: Query<(Entity, &Chunk), Without<Mesh3d>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, chunk) in &chunks {
        let mesh = chunk_build_mesh(&index, blocks, chunk.chunk);
        commands.entity(entity).insert((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
        ));
    }
}

pub fn chunk_build_mesh(index: &ChunksIndex, blocks: Query<&ChunkBlocks>, chunk: IVec3) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let entity = index.index[&chunk];
    for (&local, ()) in &blocks.get(entity).unwrap().blocks {
        let global = local_to_global(chunk, local);
        make_cube_mesh(
            local,
            &mut positions,
            &mut normals,
            &mut indices,
            Sides::NORMAL.map(|v| {
                !index
                    .get_block(|e| blocks.get(e).ok(), global + v)
                    .unwrap_or(false)
            }),
        );
    }
    Mesh::new(TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}
