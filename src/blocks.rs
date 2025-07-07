use bevy::{
    ecs::component::Component,
    math::{IVec3, Vec3Swizzles},
    platform::collections::HashMap,
};
use noisy_bevy::simplex_noise_2d;

use crate::{CHUNK_WIDTH, chunks::local_to_global};

#[derive(Component)]
pub struct ChunkBlocks {
    pub blocks: HashMap<IVec3, ()>,
}

impl ChunkBlocks {
    pub fn new(chunk: IVec3) -> Self {
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
