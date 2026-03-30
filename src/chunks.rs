use std::ops::RangeInclusive;

use crate::block::{Block, Oclusion};
use crate::chunk_blocks::ChunkBlocks;
use crate::chunk_render::NeedsRemeshing;
use crate::spacial::{Side, Sides, SidesExt};
use crate::terrain::TerrainGenerationParameters;
use crate::{CHUNK_WIDTH, octahedron};
use bevy::ecs::query::QueryEntityError;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

#[derive(Resource)]
pub struct ChunksIndex {
    index: HashMap<IVec3, Entity>,
}

pub fn chunks_setup(mut commands: Commands) {
    let terrain = TerrainGenerationParameters::new();
    commands.insert_resource(ChunksIndex::new());
    commands.insert_resource(terrain);
    commands.add_observer(observe_chunk_modify);
}

fn observe_chunk_modify(
    trigger: On<Modify>,
    index: Res<ChunksIndex>,
    mut blocks: Query<&mut ChunkBlocks>,
    mut commands: Commands,
) {
    match *trigger {
        Modify::Place { global, block } => {
            let Some((chunk, _)) = index.global_to_local(global) else {
                return;
            };
            index.set_block(|e| blocks.get_mut(e), global, block);
            commands.entity(chunk).insert(NeedsRemeshing);
            for side in Side::ALL {
                let global = side.neighbour(global);
                let Some((neighbour, _)) = index.global_to_local(global) else {
                    continue;
                };
                if neighbour == chunk {
                    continue;
                }
                if index
                    .get_block(|e| blocks.get(e), global)
                    .unwrap_or(Block::Air)
                    .oclusion()
                    != Oclusion::None
                {
                    commands.entity(neighbour).insert(NeedsRemeshing);
                }
            }
        }
    }
}

/// Chunk position on the chunk grid
///
/// The unit is the chunk size, not a block
#[derive(Component, Clone, Copy, Debug)]
pub struct Chunk {
    pub chunk: IVec3,
}

#[derive(Component, Clone, Copy)]
pub struct Loader {
    pub radius: f32,
    pub buffer: f32,
}

#[derive(Event, Debug)]
pub enum Modify {
    Place { global: IVec3, block: Block },
}

impl Loader {
    pub const ZONE_MESH: u32 = 0;
    pub const ZONE_STRUCT: u32 = 2;
    pub const ZONE_BLOCKS: u32 = 4;
    const INTER_STAGES: i32 = 4;

    pub fn new(radius: f32, buffer: f32) -> Self {
        assert!(buffer >= 1.0);
        Self { radius, buffer }
    }
    pub fn index_range(self, at: f32) -> RangeInclusive<i32> {
        let min = ((at - self.radius) / CHUNK_WIDTH as f32).floor() as i32 - Self::INTER_STAGES;
        let max = ((at + self.radius) / CHUNK_WIDTH as f32).ceil() as i32 + Self::INTER_STAGES;
        min..=max
    }
    // pub fn inside_zone(self, loader: Vec3, chunk: Chunk, zone: u32) -> bool {
    //     Self::distance(loader, chunk.center(), zone) <= self.radius
    // }
    pub fn inside_zone(self, loader: Vec3, chunk: Chunk, zone: u32) -> Option<u32> {
        let distance = Self::distance(loader, chunk.center(), zone);
        if distance <= self.radius {
            Some(distance as u32)
        } else {
            None
        }
    }
    pub fn outside_zone(self, loader: Vec3, chunk: Chunk, zone: u32) -> bool {
        Self::distance(loader, chunk.center(), zone) > self.radius + self.buffer
    }
    fn distance(lhs: Vec3, rhs: Vec3, zone: u32) -> f32 {
        octahedron::distance(lhs, zone as f32 * CHUNK_WIDTH as f32, rhs)
    }
}

impl Chunk {
    pub fn center(self) -> Vec3 {
        (self.chunk.as_vec3() + 0.5) * CHUNK_WIDTH as f32
    }
    pub fn neighbours(self) -> Sides<Self> {
        Sides::NORMAL.map(|v| self + v)
    }
}

impl From<IVec3> for Chunk {
    fn from(chunk: IVec3) -> Self {
        Self { chunk }
    }
}

impl std::ops::Add<IVec3> for Chunk {
    type Output = Chunk;

    fn add(self, rhs: IVec3) -> Chunk {
        Self {
            chunk: self.chunk + rhs,
        }
    }
}

pub fn chunk_neighbours_hexahedron(chunk: Chunk) -> [Chunk; 26] {
    [
        ivec3(-1, -1, -1),
        ivec3(-1, -1, 0),
        ivec3(-1, -1, 1),
        ivec3(-1, 0, -1),
        ivec3(-1, 0, 0),
        ivec3(-1, 0, 1),
        ivec3(-1, 1, -1),
        ivec3(-1, 1, 0),
        ivec3(-1, 1, 1),
        ivec3(0, -1, -1),
        ivec3(0, -1, 0),
        ivec3(0, -1, 1),
        ivec3(0, 0, -1),
        ivec3(0, 0, 1),
        ivec3(0, 1, -1),
        ivec3(0, 1, 0),
        ivec3(0, 1, 1),
        ivec3(1, -1, -1),
        ivec3(1, -1, 0),
        ivec3(1, -1, 1),
        ivec3(1, 0, -1),
        ivec3(1, 0, 0),
        ivec3(1, 0, 1),
        ivec3(1, 1, -1),
        ivec3(1, 1, 0),
        ivec3(1, 1, 1),
    ]
    .map(|shift| Chunk::from(chunk.chunk + shift))
}

type Queried<'a, T> = Result<&'a T, QueryEntityError>;
type QueriedMut<'a, T> = Result<Mut<'a, T>, QueryEntityError>;

impl ChunksIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }
    pub fn global_to_local(&self, global: IVec3) -> Option<(Entity, IVec3)> {
        let (chunk, local) = global_to_local(global);
        Some((self.get(chunk)?, local))
    }

    pub fn get_block<'a>(
        &self,
        blocks: impl FnOnce(Entity) -> Queried<'a, ChunkBlocks>,
        global: IVec3,
    ) -> Option<Block> {
        let (chunk, local) = self.global_to_local(global)?;
        Some(blocks(chunk).ok()?.get(local))
    }

    pub fn set_block<'a>(
        &self,
        blocks: impl FnOnce(Entity) -> QueriedMut<'a, ChunkBlocks>,
        global: IVec3,
        block: Block,
    ) {
        let Some((chunk, local)) = self.global_to_local(global) else {
            warn!("attempt to set block in non-indexed chunk");
            return;
        };
        let Some(mut blocks) = blocks(chunk).ok() else {
            warn!("attempt to set block in non-loaded chunk");
            return;
        };
        blocks.set(local, block);
    }

    pub fn set_block_if_stronger<'a>(
        &self,
        blocks: impl FnOnce(Entity) -> QueriedMut<'a, ChunkBlocks>,
        global: IVec3,
        block: Block,
    ) {
        let Some((chunk, local)) = self.global_to_local(global) else {
            warn!("attempt to set block in non-indexed chunk");
            return;
        };
        let Some(mut blocks) = blocks(chunk).ok() else {
            warn!("attempt to set block in non-loaded chunk");
            return;
        };
        if blocks.get(local) < block {
            blocks.set(local, block);
        }
    }

    pub fn get(&self, chunk: Chunk) -> Option<Entity> {
        self.index.get(&chunk.chunk).copied()
    }
}

pub fn local_to_global(chunk: Chunk, local: IVec3) -> IVec3 {
    assert_is_local(local);
    CHUNK_WIDTH * chunk.chunk + local
}

pub fn global_to_local(global: IVec3) -> (Chunk, IVec3) {
    (
        global.div_euclid(IVec3::splat(CHUNK_WIDTH)).into(),
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

pub fn chunk_indexer(
    loaders: Query<(&Transform, &Loader)>,
    mut commands: Commands,
    mut index: ResMut<ChunksIndex>,
) {
    for (transform, loader) in &loaders {
        let Vec3 { x, y, z } = transform.translation;
        for x in loader.index_range(x) {
            for y in loader.index_range(y) {
                for z in loader.index_range(z) {
                    let chunk = IVec3 { x, y, z };
                    let chunk = Chunk { chunk };
                    if index.get(chunk).is_none() {
                        let global = local_to_global(chunk, IVec3::ZERO).as_vec3();
                        let transform = Transform::from_translation(global);
                        let entity = commands
                            .spawn((chunk, transform, Visibility::default()))
                            .id();
                        index.index.insert(chunk.chunk, entity);
                    }
                }
            }
        }
    }
}

// pub fn reset_chunks(
//     mut commands: Commands,
//     mut index: ResMut<ChunksIndex>,
//     chunks: Query<Entity, With<ChunkBlocks>>,
// ) {
//     *index = ChunksIndex::new();
//     for chunk in chunks {
//         commands.entity(chunk).despawn();
//     }
// }
