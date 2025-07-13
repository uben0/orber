use crate::block::{Block, Oclusion};
use crate::chunk_blocks::ChunkBlocks;
use crate::chunk_meshing::NeedsRemeshing;
use crate::spacial::{Side, Sides, SidesExt};
use crate::{CHUNK_WIDTH, octahedron};
use bevy::ecs::query::QueryEntityError;
use bevy::prelude::*;
use bevy::{ecs::entity::Entity, platform::collections::HashMap};
use std::ops::RangeInclusive;

#[derive(Resource)]
pub struct ChunksIndex {
    index: HashMap<IVec3, Entity>,
}

#[derive(Component, Clone, Copy)]
pub struct Chunk {
    pub chunk: IVec3,
}

#[derive(Component, Clone, Copy)]
pub struct Loader {
    radius: f32,
    buffer: f32,
}

#[derive(Event, Debug)]
pub enum Modify {
    Place { global: IVec3, block: Block },
}

pub fn chunks_setup(mut commands: Commands) {
    commands.insert_resource(ChunksIndex::new());
    commands.add_observer(observe_chunk_modify);
}

fn observe_chunk_modify(
    trigger: Trigger<Modify>,
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

// pub fn chunk_state_show(
//     chunks: Query<(&Chunk, Has<ChunkBlocks>, Has<Mesh3d>)>,
//     mut gizmos: Gizmos,
// ) {
//     for (&chunk, has_blocks, has_mesh) in &chunks {
//         let color = match (has_blocks, has_mesh) {
//             (false, false) => Color::srgb(1.0, 0.0, 0.0),
//             (true, false) => Color::srgb(1.0, 1.0, 0.0),
//             (true, true) => Color::srgb(0.0, 0.0, 1.0),
//             (false, true) => panic!(),
//         };
//         gizmos.cuboid(
//             Transform {
//                 translation: chunk.center(),
//                 rotation: default(),
//                 scale: Vec3::splat(CHUNK_WIDTH as f32 - 1.0),
//             },
//             color,
//         );
//     }
// }

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
                        let entity = commands.spawn((chunk, transform)).id();
                        index.index.insert(chunk.chunk, entity);
                    }
                }
            }
        }
    }
}

impl Loader {
    pub const ZONE_MESH: u32 = 0;
    pub const ZONE_BLOCKS: u32 = 1;

    pub fn new(radius: f32, buffer: f32) -> Self {
        assert!(buffer >= 1.0);
        Self { radius, buffer }
    }
    pub fn index_range(self, at: f32) -> RangeInclusive<i32> {
        const INTER_STAGES: i32 = 1;
        let min = ((at - self.radius) / CHUNK_WIDTH as f32).floor() as i32 - INTER_STAGES;
        let max = ((at + self.radius) / CHUNK_WIDTH as f32).ceil() as i32 + INTER_STAGES;
        min..=max
    }
    pub fn inside_zone(self, loader: Vec3, chunk: Chunk, zone: u32) -> bool {
        Self::distance(loader, chunk.center(), zone) <= self.radius
    }
    pub fn inside_zone2(self, loader: Vec3, chunk: Chunk, zone: u32) -> Option<u32> {
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
