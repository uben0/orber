use bevy::prelude::*;
use bevy::{ecs::entity::Entity, platform::collections::HashMap};

use crate::CHUNK_WIDTH;
use crate::blocks::ChunkBlocks;

#[derive(Resource)]
pub struct ChunksIndex {
    pub index: HashMap<IVec3, Entity>,
}

#[derive(Component)]
pub struct Chunk {
    pub chunk: IVec3,
}

impl ChunksIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }
    pub fn global_to_local(&self, global: IVec3) -> Option<(Entity, IVec3)> {
        let (chunk, local) = global_to_local(global);
        Some((*self.index.get(&chunk)?, local))
    }

    pub fn get_block<'a>(
        &self,
        blocks: impl FnOnce(Entity) -> Option<&'a ChunkBlocks>,
        global: IVec3,
    ) -> Option<bool> {
        let (chunk, local) = self.global_to_local(global)?;
        Some(blocks(chunk)?.get(local))
    }
}

pub fn local_to_global(chunk: IVec3, local: IVec3) -> IVec3 {
    assert_is_local(local);
    CHUNK_WIDTH * chunk + local
}

pub fn global_to_local(global: IVec3) -> (IVec3, IVec3) {
    (
        global.div_euclid(IVec3::splat(CHUNK_WIDTH)),
        global.rem_euclid(IVec3::splat(CHUNK_WIDTH)),
    )
}

#[inline]
pub fn assert_is_local(local: IVec3) {
    assert!(local.x >= 0);
    assert!(local.y >= 0);
    assert!(local.z >= 0);
    assert!(local.x < CHUNK_WIDTH);
    assert!(local.y < CHUNK_WIDTH);
    assert!(local.z < CHUNK_WIDTH);
}

pub fn chunk_indexer(mut commands: Commands, mut index: ResMut<ChunksIndex>) {
    for x in -2..=2 {
        for y in -2..=2 {
            for z in -2..=2 {
                let chunk = IVec3 { x, y, z };
                if !index.index.contains_key(&chunk) {
                    let global = local_to_global(chunk, IVec3::ZERO).as_vec3();
                    let transform = Transform::from_translation(global);
                    let entity = commands
                        .spawn((Chunk { chunk }, transform, ChunkBlocks::new(chunk)))
                        .id();
                    index.index.insert(chunk, entity);
                }
            }
        }
    }
}

#[test]
fn test_global_local_transitions() {
    for x in -100..100 {
        for y in -100..100 {
            for z in -100..100 {
                let global = IVec3 { x, y, z };
                let (chunk, local) = global_to_local(global);
                let global_rebuilt = local_to_global(chunk, local);
                assert_eq!(global, global_rebuilt);
            }
        }
    }
}
