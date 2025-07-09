use crate::{
    CHUNK_WIDTH,
    chunks::{Chunk, Loader, local_to_global},
};
use bevy::{math::Vec3Swizzles, platform::collections::HashMap, prelude::*};
use noisy_bevy::simplex_noise_2d;

#[derive(Component)]
pub struct ChunkBlocks {
    pub blocks: HashMap<IVec3, ()>,
}

pub fn chunk_generation(
    chunks: Query<(Entity, &Chunk), Without<ChunkBlocks>>,
    loaders: Query<(&Transform, &Loader)>,
    mut commands: Commands,
) {
    for (entity, &chunk) in &chunks {
        if loaders.iter().any(|(transform, &loader)| {
            loader.inside_zone(transform.translation, chunk, Loader::ZONE_BLOCKS)
        }) {
            commands.entity(entity).insert(ChunkBlocks::new(chunk));
        }
    }
}

impl ChunkBlocks {
    pub fn new(chunk: Chunk) -> Self {
        let mut blocks = HashMap::new();
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                let local = IVec3 { x, y: 0, z };
                let global = local_to_global(chunk, local);

                let elevation = (simplex_noise_2d(global.xz().as_vec2() / 20.0) * 5.0) as i32;
                for y in 0..CHUNK_WIDTH {
                    let local = IVec3 { x, y, z };
                    let global = local_to_global(chunk, local);
                    if global.y > elevation {
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
