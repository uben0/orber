use crate::{
    chunk_blocks::ChunkBlocks, chunks::ChunksIndex, ray_travel::RayTraveler, spacial::Side,
};
use bevy::prelude::*;

pub struct BlockPointingPlugin;

// TODO: seperate Pointing as a component
#[derive(Component)]
pub struct BlockPointer {
    pub range: f32,
    pub pointing: Option<Pointing>,
}

#[derive(Debug, Clone, Copy)]
pub struct Pointing {
    pub global: IVec3,
    pub side: Side,
}

impl BlockPointer {
    pub fn new(range: f32) -> Self {
        Self {
            range,
            pointing: None,
        }
    }
}

impl Plugin for BlockPointingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_gizmo_config(
            PointedBlockOverlay,
            GizmoConfig {
                line: GizmoLineConfig {
                    width: 2.0,
                    ..default()
                },
                depth_bias: -0.001,
                ..default()
            },
        )
        .add_systems(
            Update,
            (pointed_block, pointed_block_overlay.after(pointed_block)),
        );
    }
}

#[derive(GizmoConfigGroup, Default, Reflect)]
struct PointedBlockOverlay;

fn pointed_block(
    pointers: Query<(&Transform, &mut BlockPointer)>,
    blocks: Query<&ChunkBlocks>,
    index: Res<ChunksIndex>,
) {
    for (transform, mut pointer) in pointers {
        pointer.pointing = RayTraveler::new(
            transform.translation,
            transform.rotation * Dir3::NEG_Z,
            pointer.range,
        )
        .find(|step| index.get_block(|e| blocks.get(e).ok(), step.voxel) == Some(true))
        .map(|found| Pointing {
            global: found.voxel,
            side: found.side,
        });
    }
}

fn pointed_block_overlay(pointers: Query<&BlockPointer>, mut gizmos: Gizmos<PointedBlockOverlay>) {
    for pointer in pointers {
        if let Some(pointing) = pointer.pointing {
            gizmos.cuboid(
                Transform::from_translation(pointing.global.as_vec3() + 0.5 * Vec3::ONE),
                Color::srgb(0.0, 0.0, 0.0),
            );
        }
    }
}
