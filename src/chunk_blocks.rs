use crate::CHUNK_WIDTH;
use crate::array_queue::ArrayVecExt;
use crate::block::Block;
use crate::chunk_render::NeedsRemeshing;
use crate::chunks::{Chunk, ChunksIndex, Loader, chunk_neighbours_hexahedron, local_to_global};
use crate::terrain::TerrainGenerationParameters;
use arrayvec::ArrayVec;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use std::hash::Hash;
use std::hash::{DefaultHasher, Hasher};

#[derive(Component)]
pub struct GenStage;

#[derive(Component)]
pub struct ChunkBlocks {
    default: Block,
    blocks: HashMap<IVec3, Block>,
}

const MAX_CHUNK_GEN_PER_FRAME: usize = 2;

pub fn chunk_generation(
    parameters: Res<TerrainGenerationParameters>,
    chunks: Query<(Entity, &Chunk), Without<ChunkBlocks>>,
    loaders: Query<(&Transform, &Loader)>,
    mut commands: Commands,
) {
    let mut mins: ArrayVec<(u32, Entity, Chunk), { MAX_CHUNK_GEN_PER_FRAME }> = ArrayVec::new();
    for (entity, &chunk) in &chunks {
        if let Some(distance) = loaders
            .iter()
            .filter_map(|(transform, &loader)| {
                loader.inside_zone(transform.translation, chunk, Loader::ZONE_BLOCKS)
            })
            .min()
        {
            mins.insert_by_key((distance, entity, chunk), |&(d, _, _)| d);
        }
    }
    for (_, entity, chunk) in mins {
        commands
            .entity(entity)
            .insert((ChunkBlocks::new(chunk, &parameters), GenStage));
    }
}

pub fn chunk_generation_struct(
    candidates: Query<(Entity, &Chunk), With<GenStage>>,
    mut blocks: Query<&mut ChunkBlocks>,
    index: Res<ChunksIndex>,
    loaders: Query<(&Transform, &Loader)>,
    mut commands: Commands,
) {
    for (candidate, &chunk) in candidates {
        if loaders.iter().any(|(transform, &loader)| {
            loader
                .inside_zone(transform.translation, chunk, Loader::ZONE_STRUCT)
                .is_some()
        }) {
            if chunk_neighbours_hexahedron(chunk)
                .iter()
                .all(|&n| index.get(n).map(|e| blocks.contains(e)).unwrap_or(false))
            {
                commands
                    .entity(candidate)
                    .remove::<GenStage>()
                    .insert(NeedsRemeshing);
                for x in 0..CHUNK_WIDTH {
                    for y in 0..CHUNK_WIDTH {
                        for z in 0..CHUNK_WIDTH {
                            let global = local_to_global(chunk, ivec3(x, y, z));
                            if index.get_block(|e| blocks.get(e), global) == Some(Block::Grass)
                                && index.get_block(|e| blocks.get(e), global + IVec3::Y)
                                    == Some(Block::Air)
                                && seeded_random(global) > 0.97
                            {
                                build_tree(|relative, block| {
                                    index.set_block_if_stronger(
                                        |e| blocks.get_mut(e),
                                        global + relative,
                                        block,
                                    );
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

fn build_tree(mut place: impl FnMut(IVec3, Block)) {
    place(ivec3(0, 0, 0), Block::Log);
    place(ivec3(0, 1, 0), Block::Log);
    place(ivec3(0, 2, 0), Block::Log);
    place(ivec3(0, 3, 0), Block::Log);

    place(ivec3(1, 3, 0), Block::Leaves);
    place(ivec3(1, 3, 1), Block::Leaves);
    place(ivec3(0, 3, 1), Block::Leaves);
    place(ivec3(-1, 3, 1), Block::Leaves);
    place(ivec3(-1, 3, 0), Block::Leaves);
    place(ivec3(-1, 3, -1), Block::Leaves);
    place(ivec3(0, 3, -1), Block::Leaves);
    place(ivec3(1, 3, -1), Block::Leaves);

    place(ivec3(1, 4, 0), Block::Leaves);
    place(ivec3(1, 4, 1), Block::Leaves);
    place(ivec3(0, 4, 1), Block::Leaves);
    place(ivec3(-1, 4, 1), Block::Leaves);
    place(ivec3(-1, 4, 0), Block::Leaves);
    place(ivec3(-1, 4, -1), Block::Leaves);
    place(ivec3(0, 4, -1), Block::Leaves);
    place(ivec3(1, 4, -1), Block::Leaves);
    place(ivec3(0, 4, 0), Block::Leaves);

    place(ivec3(0, 5, 0), Block::Leaves);
}

fn seeded_random<T: Hash>(seed: T) -> f32 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    SmallRng::seed_from_u64(hasher.finish()).random()
}

impl ChunkBlocks {
    pub const AIR: Self = Self {
        default: Block::Air,
        blocks: HashMap::new(),
    };
    pub fn new(chunk: Chunk, parameters: &TerrainGenerationParameters) -> Self {
        let mut blocks = Self::AIR;
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                let local = IVec3 { x, y: 0, z };
                let global = local_to_global(chunk, local);

                let terrain = parameters.descriptor(global.xz());
                for y in 0..CHUNK_WIDTH {
                    let local = IVec3 { x, y, z };
                    let global = local_to_global(chunk, local);

                    let block = if global.y < terrain.elevation.round() as i32 {
                        Block::Stone
                    } else if global.y < (terrain.elevation + terrain.sediment).round() as i32 {
                        if terrain.continent > 2.0 {
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
    // pub fn assert_consistency(&self) {
    //     for (&local, &block) in &self.blocks {
    //         assert_is_local(local);
    //         assert_ne!(block, self.default);
    //     }
    // }
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
