use crate::spacial::{Axis, Side};
use arrayvec::ArrayVec;
use bevy::math::{Dir3, IVec3, Vec3};
use std::cmp::Ordering;

pub struct RayTraveler {
    axis_travelers: ArrayVec<AxisTraveler, 3>,
    time_current: f32,
    time_limit: f32,
    ray_origin: Vec3,
    ray_vector: Dir3,
    voxel_current: IVec3,
}

pub struct Step {
    pub side: Side,
    pub voxel: IVec3,
    pub position: Vec3,
    pub time: f32,
}

struct AxisTraveler {
    next: f32,
    step: f32,
    dir: Side,
}

impl RayTraveler {
    pub fn new(origin: Vec3, ray: Dir3, limit: f32) -> Self {
        Self {
            axis_travelers: [
                (origin.x, ray.x, Axis::X),
                (origin.y, ray.y, Axis::Y),
                (origin.z, ray.z, Axis::Z),
            ]
            .into_iter()
            .filter_map(|(origin, ray, axis)| match ray.partial_cmp(&0.0)? {
                Ordering::Less => Some(AxisTraveler {
                    // TODO: make symetric
                    next: ((origin + 0.0) - origin.floor()) / ray.abs(),
                    step: 1.0 / ray.abs(),
                    dir: axis.negative(),
                }),
                Ordering::Equal => None,
                Ordering::Greater => Some(AxisTraveler {
                    next: ((origin + 1.0).floor() - origin) / ray.abs(),
                    step: 1.0 / ray.abs(),
                    dir: axis.positive(),
                }),
            })
            .collect(),
            time_current: 0.0,
            time_limit: limit,
            ray_origin: origin,
            ray_vector: ray,
            voxel_current: origin.floor().as_ivec3(),
        }
    }
}

impl Iterator for RayTraveler {
    type Item = Step;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: check is redoundant
        if self.time_current > self.time_limit {
            return None;
        }
        let axis_traveler = self
            .axis_travelers
            .iter_mut()
            .min_by(|lhs, rhs| lhs.next.partial_cmp(&rhs.next).unwrap())?;
        self.time_current = axis_traveler.next;
        if self.time_current > self.time_limit {
            return None;
        }

        axis_traveler.next += axis_traveler.step;
        self.voxel_current = axis_traveler.dir.neighbour(self.voxel_current);
        Some(Step {
            side: axis_traveler.dir.oposite(),
            voxel: self.voxel_current,
            position: self.ray_origin + self.ray_vector * self.time_current,
            time: self.time_current,
        })
    }
}
