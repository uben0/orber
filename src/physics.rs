use bevy::prelude::*;

use crate::{
    chunk_blocks::ChunkBlocks,
    chunks::ChunksIndex,
    ray_travel::RayTraveler,
    spacial::{AxisSplit, Side},
};

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

#[derive(Component)]
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

fn damp_velocity(collider: Query<(&mut Velocity, Has<Grounded>), With<Collider>>, time: Res<Time>) {
    for (mut velocity, grounded) in collider {
        let rate: f32 = if grounded { 0.7 } else { 0.9 };
        velocity.linear.x *= rate.powf(time.delta_secs() + 1.0);
        velocity.linear.z *= rate.powf(time.delta_secs() + 1.0);
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
                let dim = step.side.axis();

                let (_, [plane_u, plane_v]) = (step.position - corner_select).split(dim);
                let (_, [size_u, size_v]) = cl.size.split(dim);

                // on the UV plane, we select all voxels covered by the side of the collider
                for u in plane_u.floor() as i32..=(plane_u + size_u).floor() as i32 {
                    for v in plane_v.floor() as i32..=(plane_v + size_v).floor() as i32 {
                        // we find the global coordinate of each voxel
                        let selected = IVec3::compose(dim, step.voxel[dim], [u, v]);

                        // if a block is present, a collision occur
                        if index.get_block(|e| blocks.get(e), selected) == Some(true) {
                            // we correct the vector component to stop at the collision
                            shift[dim] *= step.time / length;
                            // we stop slightly before the collision
                            shift[dim] -= dir[dim].signum() * 1e-4;
                            // the collision absorbs all kinetic energy
                            vl.linear[dim] = 0.0;

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

        if index.get_block(
            |e| blocks.get(e),
            (corner_active + shift).floor().as_ivec3(),
        ) == Some(true)
        {
            println!("collider tunneling");
            println!(" - pos    {:.10}", corner_active);
            println!(" - shift* {:.10}", shift);
            println!(" - pos*   {:.10}", corner_active + shift);
            println!();
            commands.entity(entity).remove::<Velocity>();
            return;
        }

        tr.translation += shift;
    }
}
