use crate::block::Block;
use crate::chunk_blocks::ChunkBlocks;
use crate::chunks::ChunksIndex;
use crate::ray_travel::RayTraveler;
use crate::spacial::{Side, Sign, Vec3Ext};
use bevy::math::bounding::{Aabb3d, IntersectsVolume};
use bevy::prelude::*;

#[derive(SystemSet, Clone, PartialEq, Eq, Debug, Hash)]
pub struct ApplyPhysics;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (apply_gravity, damp_velocity, apply_velocity)
                .chain()
                .in_set(ApplyPhysics),
        );
    }
}

#[derive(Component, Clone, Copy)]
pub struct Collider {
    pub size: Vec3,
    pub anchor: Vec3,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Velocity {
    pub linear: Vec3,
}

#[derive(Component)]
pub struct Grounded;

fn collider_aabb(transform: Transform, collider: Collider) -> Aabb3d {
    Aabb3d {
        min: (transform.translation - collider.anchor).into(),
        max: (transform.translation - collider.anchor + collider.size).into(),
    }
}

fn global_aabb(global: IVec3) -> Aabb3d {
    Aabb3d {
        min: global.as_vec3a(),
        max: (global + IVec3::ONE).as_vec3a(),
    }
}

pub fn intersects(transform: Transform, collider: Collider, global: IVec3) -> bool {
    collider_aabb(transform, collider).intersects(&global_aabb(global))
}

fn damp_velocity(collider: Query<(&mut Velocity, Has<Grounded>), With<Collider>>, time: Res<Time>) {
    let rate_grounded = 1e-9f32.powf(time.delta_secs());
    let rate_airborne = 1e-3f32.powf(time.delta_secs());

    for (mut velocity, grounded) in collider {
        let rate = match grounded {
            true => rate_grounded,
            false => rate_airborne,
        };
        velocity.linear.x *= rate;
        velocity.linear.z *= rate;
    }
}

fn apply_gravity(velocity: Query<&mut Velocity>, time: Res<Time>) {
    for mut velocity in velocity {
        velocity.linear += Vec3::NEG_Y * 40.0 * time.delta_secs();
    }
}

fn apply_velocity(
    index: Res<ChunksIndex>,
    blocks: Query<&ChunkBlocks>,
    collider: Query<(Entity, &mut Transform, &Collider, &mut Velocity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut tr, cl, mut vl) in collider {
        // which side of the collider is advancing
        let corner_select = Vec3 {
            x: if vl.linear.x < 0.0 { 0.0 } else { cl.size.x },
            y: if vl.linear.y < 0.0 { 0.0 } else { cl.size.y },
            z: if vl.linear.z < 0.0 { 0.0 } else { cl.size.z },
        };
        let corner_active = tr.translation - cl.anchor + corner_select;

        // the current translation
        let mut shift = vl.linear * time.delta_secs();

        let mut grounded = false;

        'search: while let Ok(dir) = shift.try_into() {
            let length = shift.length();
            for step in RayTraveler::new(corner_active, dir, length) {
                // to avoid code duplication, each symetric situation through dimension permutation is made identic by a reversible swizzle
                let axis = step.side.axis();

                let (_, [plane_u, plane_v]) =
                    (step.position - corner_select).split(axis, Sign::Pos);
                let (_, [size_u, size_v]) = cl.size.split(axis, Sign::Pos);

                // on the UV plane, we select all voxels covered by the side of the collider
                for u in plane_u.floor() as i32..=(plane_u + size_u).floor() as i32 {
                    for v in plane_v.floor() as i32..=(plane_v + size_v).floor() as i32 {
                        // we find the global coordinate of each voxel
                        let selected = IVec3::compose(axis, Sign::Pos, step.voxel[axis], [u, v]);

                        // if a block is present, a collision occur
                        if index
                            .get_block(|e| blocks.get(e), selected)
                            .unwrap_or(Block::Air)
                            .collides()
                        {
                            // we correct the vector component to stop at the collision
                            shift[axis] *= step.time / length;
                            // we stop slightly before the collision
                            shift[axis] -= dir[axis].signum() * 1e-4;
                            // the collision absorbs all kinetic energy
                            vl.linear[axis] = 0.0;

                            if step.side == Side::YPos {
                                grounded = true;
                            }

                            // we restart the collision search with the corrected shift
                            continue 'search;
                        }
                    }
                }
            }
            // no more collisions are detected
            break 'search;
        }

        if grounded {
            commands.entity(entity).insert_if_new(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }

        // if index
        //     .get_block(
        //         |e| blocks.get(e),
        //         (corner_active + shift).floor().as_ivec3(),
        //     )
        //     .unwrap_or(Block::Air)
        //     .collides()
        // {
        //     println!("collider tunneling");
        //     println!(" - pos    {:.10}", corner_active);
        //     println!(" - shift* {:.10}", shift);
        //     println!(" - pos*   {:.10}", corner_active + shift);
        //     println!();
        //     return;
        // }

        tr.translation += shift;
    }
}
