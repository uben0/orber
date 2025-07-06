use std::ops::Index;

use crate::swizzle::Swizzle3;

#[derive(Debug, Clone, Copy)]
pub enum Side {
    XPos,
    XNeg,
    YPos,
    YNeg,
    ZPos,
    ZNeg,
}

pub struct Sides<T> {
    pub x_pos: T,
    pub x_neg: T,
    pub y_pos: T,
    pub y_neg: T,
    pub z_pos: T,
    pub z_neg: T,
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

impl Side {
    pub const ALL: [Self; 6] = [
        Self::XPos,
        Self::XNeg,
        Self::YPos,
        Self::YNeg,
        Self::ZPos,
        Self::ZNeg,
    ];
    /// A quadruple of points forming a clockwise square
    pub fn quad(self) -> [[f32; 3]; 4] {
        let (swap, depth) = match self {
            Side::XPos => (Swizzle3::XYZ, 1.0),
            Side::XNeg => (Swizzle3::XZY, 0.0),
            Side::YPos => (Swizzle3::YZX, 1.0),
            Side::YNeg => (Swizzle3::YXZ, 0.0),
            Side::ZPos => (Swizzle3::ZXY, 1.0),
            Side::ZNeg => (Swizzle3::ZYX, 0.0),
        };
        [
            swap * [depth, 0.0, 0.0],
            swap * [depth, 1.0, 0.0],
            swap * [depth, 1.0, 1.0],
            swap * [depth, 0.0, 1.0],
        ]
    }
    pub fn normal(self) -> [f32; 3] {
        match self {
            Side::XPos => [1.0, 0.0, 0.0],
            Side::XNeg => [-1.0, 0.0, 0.0],
            Side::YPos => [0.0, 1.0, 0.0],
            Side::YNeg => [0.0, -1.0, 0.0],
            Side::ZPos => [0.0, 0.0, 1.0],
            Side::ZNeg => [0.0, 0.0, -1.0],
        }
    }
}
