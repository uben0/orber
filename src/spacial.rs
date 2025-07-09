use crate::swizzle::Swizzle3;
use bevy::math::{IVec3, Vec3};
use std::ops::{AddAssign, Index};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    XPos,
    XNeg,
    YPos,
    YNeg,
    ZPos,
    ZNeg,
}

#[derive(Debug, Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

pub struct Sides<T> {
    pub x_pos: T,
    pub x_neg: T,
    pub y_pos: T,
    pub y_neg: T,
    pub z_pos: T,
    pub z_neg: T,
}

impl<T> Index<Side> for [T; 6] {
    type Output = T;

    fn index(&self, index: Side) -> &Self::Output {
        match index {
            Side::XPos => self.index(0),
            Side::XNeg => self.index(1),
            Side::YPos => self.index(2),
            Side::YNeg => self.index(3),
            Side::ZPos => self.index(4),
            Side::ZNeg => self.index(5),
        }
    }
}

impl<T> Index<Side> for Sides<T> {
    type Output = T;

    fn index(&self, index: Side) -> &Self::Output {
        match index {
            Side::XPos => &self.x_pos,
            Side::XNeg => &self.x_neg,
            Side::YPos => &self.y_pos,
            Side::YNeg => &self.y_neg,
            Side::ZPos => &self.z_pos,
            Side::ZNeg => &self.z_neg,
        }
    }
}

impl<T> Sides<T> {
    pub fn map<U>(self, mut m: impl FnMut(T) -> U) -> Sides<U> {
        Sides {
            x_pos: m(self.x_pos),
            x_neg: m(self.x_neg),
            y_pos: m(self.y_pos),
            y_neg: m(self.y_neg),
            z_pos: m(self.z_pos),
            z_neg: m(self.z_neg),
        }
    }
    pub fn all(self, p: impl FnMut(T) -> bool) -> bool {
        [
            self.x_pos, self.x_neg, self.y_pos, self.y_neg, self.z_pos, self.z_neg,
        ]
        .into_iter()
        .all(p)
    }
}

pub trait SidesExt<T> {
    const NORMAL: Sides<T>;
}

impl SidesExt<Vec3> for Sides<Vec3> {
    const NORMAL: Sides<Vec3> = Sides {
        x_pos: Vec3::X,
        x_neg: Vec3::NEG_X,
        y_pos: Vec3::Y,
        y_neg: Vec3::NEG_Y,
        z_pos: Vec3::Z,
        z_neg: Vec3::NEG_Z,
    };
}
impl SidesExt<IVec3> for Sides<IVec3> {
    const NORMAL: Sides<IVec3> = Sides {
        x_pos: IVec3::X,
        x_neg: IVec3::NEG_X,
        y_pos: IVec3::Y,
        y_neg: IVec3::NEG_Y,
        z_pos: IVec3::Z,
        z_neg: IVec3::NEG_Z,
    };
}

impl Side {
    pub const ALL: [Self; 6] = [
        Self::XPos,
        Self::XNeg,
        Self::YPos,
        Self::YNeg,
        Self::ZPos,
        Self::ZNeg,
    ];
    pub fn neighbour(self, of: IVec3) -> IVec3 {
        Sides::<IVec3>::NORMAL[self] + of
    }
    pub const fn oposite(self) -> Self {
        match self {
            Side::XPos => Side::XNeg,
            Side::XNeg => Side::XPos,
            Side::YPos => Side::YNeg,
            Side::YNeg => Side::YPos,
            Side::ZPos => Side::ZNeg,
            Side::ZNeg => Side::ZPos,
        }
    }
    /// A quadruple of points forming a clockwise square
    pub fn quad(self) -> [[f32; 3]; 4] {
        let (swap, depth) = match self {
            Side::XPos => (Swizzle3::XYZ, 1.0), // shift 0, clockwise
            Side::XNeg => (Swizzle3::XZY, 0.0), // shift 0, counter clockwise
            Side::YPos => (Swizzle3::ZXY, 1.0), // shift 1, clockwise
            Side::YNeg => (Swizzle3::YXZ, 0.0), // shift 1, counter clockwise
            Side::ZPos => (Swizzle3::YZX, 1.0), // shift 2, clockwise
            Side::ZNeg => (Swizzle3::ZYX, 0.0), // shift 2, counter clockwise
        };
        [
            swap * [depth, 0.0, 0.0],
            swap * [depth, 1.0, 0.0],
            swap * [depth, 1.0, 1.0],
            swap * [depth, 0.0, 1.0],
        ]
    }
    pub fn normal(self) -> [f32; 3] {
        Sides::<Vec3>::NORMAL[self].into()
    }
}

impl Axis {
    pub const fn negative(self) -> Side {
        match self {
            Axis::X => Side::XNeg,
            Axis::Y => Side::YNeg,
            Axis::Z => Side::ZNeg,
        }
    }
    pub const fn positive(self) -> Side {
        match self {
            Axis::X => Side::XPos,
            Axis::Y => Side::YPos,
            Axis::Z => Side::ZPos,
        }
    }
}
