use bevy::{ecs::component::Component, math::IVec3, platform::collections::HashMap};

use crate::CHUNK_WIDTH;

#[derive(Component)]
pub struct ChunkBlocks {
    pub blocks: HashMap<IVec3, ()>,
}

impl ChunkBlocks {
    pub fn new() -> Self {
        let mut blocks = HashMap::new();
        blocks.insert(IVec3::ZERO, ());
        // blocks.insert(IVec3::X, ());
        // blocks.insert(IVec3::Z, ());

        // for x in 0..CHUNK_WIDTH {
        //     for z in 0..CHUNK_WIDTH {
        //         blocks.insert(IVec3 { x, y: 0, z }, ());
        //         // if (x + z) % 2 == 0 {
        //         //     blocks.insert(IVec3 { x, y: 1, z }, ());
        //         // }
        //     }
        // }
        Self { blocks }
    }
    pub fn get(&self, local: IVec3) -> bool {
        self.blocks.contains_key(&local)
    }
}
