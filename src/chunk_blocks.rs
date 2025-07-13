use crate::{
    CHUNK_WIDTH,
    array_queue::ArrayVecExt,
    chunks::{Chunk, Loader, local_to_global},
    terrain::TerrainDescriptor,
};
use arrayvec::ArrayVec;
use bevy::{math::Vec3Swizzles, platform::collections::HashMap, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Air,
    Stone,
    Sand,
}

// TODO: make the most aboundant block the default
//       in the sky, it is air
//       in the ground it is stone
//       in the ocean it is water
#[derive(Component)]
pub struct ChunkBlocks {
    default: Block,
    blocks: HashMap<IVec3, Block>,
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
    pub const AIR: Self = Self {
        default: Block::Air,
        blocks: HashMap::new(),
    };
    pub fn new(chunk: Chunk) -> Self {
        let mut blocks = Self::AIR;
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                let local = IVec3 { x, y: 0, z };
                let global = local_to_global(chunk, local);

                let terrain = TerrainDescriptor::at(global.xz());
                // let elevation = (simplex_noise_2d(global.xz().as_vec2() / 20.0) * 5.0) as i32;
                for y in 0..CHUNK_WIDTH {
                    let local = IVec3 { x, y, z };
                    let global = local_to_global(chunk, local);

                    let block = if global.y < terrain.elevation.round() as i32 {
                        Block::Stone
                    } else if global.y < (terrain.elevation + terrain.sediment).round() as i32 {
                        Block::Sand
                    } else {
                        Block::Air
                    };
                    blocks.set(local, block);
                }
            }
        }
        blocks
    }
    pub fn get(&self, local: IVec3) -> Block {
        self.blocks.get(&local).copied().unwrap_or(self.default)
    }
    pub fn set(&mut self, local: IVec3, block: Block) {
        if block == self.default {
            self.blocks.remove(&local);
        } else {
            self.blocks.insert(local, block);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Oclusion {
    None,
    Full,
}

impl Block {
    pub const fn oclusion(self) -> Oclusion {
        match self {
            Block::Air => Oclusion::None,
            Block::Stone | Block::Sand => Oclusion::Full,
        }
    }
    pub const fn collides(self) -> bool {
        match self {
            Block::Air => false,
            Block::Stone | Block::Sand => true,
        }
    }
}
