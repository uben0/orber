use crate::{
    CHUNK_WIDTH,
    array_queue::ArrayVecExt,
    chunks::{Chunk, Loader, local_to_global},
    terrain::TerrainDescriptor,
};
use arrayvec::ArrayVec;
use bevy::{math::Vec3Swizzles, platform::collections::HashMap, prelude::*};
use noisy_bevy::simplex_noise_2d;

#[derive(Component)]
pub struct ChunkBlocks {
    pub blocks: HashMap<IVec3, ()>,
}

const MAX_CHUNK_GEN_PER_FRAME: usize = 4;

pub fn chunk_generation(
    chunks: Query<(Entity, &Chunk), Without<ChunkBlocks>>,
    loaders: Query<(&Transform, &Loader)>,
    mut commands: Commands,
) {
    let mut mins: ArrayVec<(u32, Entity, Chunk), { MAX_CHUNK_GEN_PER_FRAME }> = ArrayVec::new();
    for (entity, &chunk) in &chunks {
        if let Some(distance) = loaders
            .iter()
            .filter_map(|(transform, &loader)| {
                loader.inside_zone2(transform.translation, chunk, Loader::ZONE_BLOCKS)
            })
            .min()
        {
            mins.insert_by_key((distance, entity, chunk), |&(d, _, _)| d);
        }
    }
    for (_, entity, chunk) in mins {
        commands.entity(entity).insert(ChunkBlocks::new(chunk));
    }
}

impl ChunkBlocks {
    pub fn new(chunk: Chunk) -> Self {
        let mut blocks = HashMap::new();
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                let local = IVec3 { x, y: 0, z };
                let global = local_to_global(chunk, local);

                let terrain = TerrainDescriptor::at(global.xz());
                // let elevation = (simplex_noise_2d(global.xz().as_vec2() / 20.0) * 5.0) as i32;
                for y in 0..CHUNK_WIDTH {
                    let local = IVec3 { x, y, z };
                    let global = local_to_global(chunk, local);
                    if global.y > terrain.continent as i32 {
                        break;
                    }
                    blocks.insert(local, ());
                }
            }
        }
        Self { blocks }
    }
    pub fn get(&self, local: IVec3) -> bool {
        self.blocks.contains_key(&local)
    }
}
