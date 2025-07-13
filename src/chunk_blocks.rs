use crate::CHUNK_WIDTH;
use crate::array_queue::ArrayVecExt;
use crate::block::Block;
use crate::chunks::{Chunk, Loader, assert_is_local, local_to_global};
use crate::terrain::TerrainDescriptor;
use arrayvec::ArrayVec;
use bevy::{math::Vec3Swizzles, platform::collections::HashMap, prelude::*};

#[derive(Component)]
pub struct ChunkBlocks {
    default: Block,
    blocks: HashMap<IVec3, Block>,
}

const MAX_CHUNK_GEN_PER_FRAME: usize = 2;

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
                for y in 0..CHUNK_WIDTH {
                    let local = IVec3 { x, y, z };
                    let global = local_to_global(chunk, local);

                    let block = if global.y < terrain.elevation.round() as i32 {
                        Block::Stone
                    } else if global.y < (terrain.elevation + terrain.sediment).round() as i32 {
                        if terrain.continent > 3.0 {
                            Block::Grass
                        } else {
                            Block::Sand
                        }
                    } else if global.y < 0 {
                        Block::Water
                    } else {
                        Block::Air
                    };
                    blocks.set(local, block);
                }
            }
        }
        blocks.select_best_default();
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
    pub fn select_best_default(&mut self) {
        let abundant = self.most_abundant();
        self.set_default(abundant);
    }
    pub fn set_default(&mut self, new_default: Block) {
        if new_default == self.default {
            return;
        }
        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_WIDTH {
                    let local = IVec3 { x, y, z };
                    match self.blocks.get(&local) {
                        Some(&block) => {
                            if block == new_default {
                                self.blocks.remove(&local);
                            }
                        }
                        None => {
                            self.blocks.insert(local, self.default);
                        }
                    }
                }
            }
        }
        self.default = new_default;
    }
    pub fn assert_consistency(&self) {
        for (&local, &block) in &self.blocks {
            assert_is_local(local);
            assert_ne!(block, self.default);
        }
    }
    fn most_abundant(&self) -> Block {
        let mut counts = HashMap::new();
        let default_count = CHUNK_WIDTH.pow(3) - self.blocks.len() as i32;
        counts.insert(self.default, default_count);
        for (_, &block) in &self.blocks {
            *counts.entry(block).or_insert(0) += 1;
        }
        let (&block, _) = counts.iter().max_by_key(|t| t.1).unwrap();
        block
    }
}
