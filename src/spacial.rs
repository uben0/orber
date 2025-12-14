use bevy::math::{IVec2, IVec3, Vec2, Vec3};
use std::{
    array::IntoIter,
    ops::{Index, IndexMut},
};

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

#[derive(Debug, Clone, Copy)]
pub enum AxisSwap {
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZXY,
    ZYX,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symetry2 {
    pub swap_xy: Sign,
    pub flip_x: Sign,
    pub flip_y: Sign,
}

impl Symetry2 {
    pub const PPP: Self = Self {
        swap_xy: Sign::Pos,
        flip_x: Sign::Pos,
        flip_y: Sign::Pos,
    };
    // pub const PPN: Self = Self {
    //     swap_xy: Sign::Pos,
    //     flip_x: Sign::Pos,
    //     flip_y: Sign::Neg,
    // };
    // pub const PNP: Self = Self {
    //     swap_xy: Sign::Pos,
    //     flip_x: Sign::Neg,
    //     flip_y: Sign::Pos,
    // };
    // pub const PNN: Self = Self {
    //     swap_xy: Sign::Pos,
    //     flip_x: Sign::Neg,
    //     flip_y: Sign::Neg,
    // };
    // pub const NPP: Self = Self {
    //     swap_xy: Sign::Neg,
    //     flip_x: Sign::Pos,
    //     flip_y: Sign::Pos,
    // };
    // pub const NPN: Self = Self {
    //     swap_xy: Sign::Neg,
    //     flip_x: Sign::Pos,
    //     flip_y: Sign::Neg,
    // };
    pub const NNP: Self = Self {
        swap_xy: Sign::Neg,
        flip_x: Sign::Neg,
        flip_y: Sign::Pos,
    };
    // pub const NNN: Self = Self {
    //     swap_xy: Sign::Neg,
    //     flip_x: Sign::Neg,
    //     flip_y: Sign::Neg,
    // };

    pub fn apply<T>(self, vec: T) -> T
    where
        T: Vec2Ext<f32>,
    {
        let [x, y] = vec.into();
        let [x, y] = match self.swap_xy {
            Sign::Pos => [x, y],
            Sign::Neg => [y, x],
        };
        let x = match self.flip_x {
            Sign::Pos => 0.0 + x,
            Sign::Neg => 1.0 - x,
        };
        let y = match self.flip_y {
            Sign::Pos => 0.0 + y,
            Sign::Neg => 1.0 - y,
        };
        [x, y].into()
    }
}

impl AxisSwap {
    pub const fn inverse(self) -> Self {
        match self {
            AxisSwap::XYZ => AxisSwap::XYZ,
            AxisSwap::XZY => AxisSwap::XZY,
            AxisSwap::YXZ => AxisSwap::YXZ,
            AxisSwap::YZX => AxisSwap::ZXY,
            AxisSwap::ZXY => AxisSwap::YZX,
            AxisSwap::ZYX => AxisSwap::ZYX,
        }
    }
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
    pub fn list(self) -> IntoIter<T, 6> {
        [
            self.x_pos, self.x_neg, self.y_pos, self.y_neg, self.z_pos, self.z_neg,
        ]
        .into_iter()
    }
    pub const fn all(elem: T) -> Self
    where
        T: Copy,
    {
        Sides {
            x_pos: elem,
            x_neg: elem,
            y_pos: elem,
            y_neg: elem,
            z_pos: elem,
            z_neg: elem,
        }
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

pub const QUAD_UV: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
pub const QUAD_INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];

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
    pub const fn sign(self) -> Sign {
        match self {
            Side::XPos | Side::YPos | Side::ZPos => Sign::Pos,
            Side::XNeg | Side::YNeg | Side::ZNeg => Sign::Neg,
        }
    }
    /// A quadruple of points forming a clockwise square
    pub fn quad<T>(self) -> [T; 4]
    where
        T: Vec3Ext<f32>,
    {
        let sign = self.sign();
        let axis = self.axis();
        let depth = match sign {
            Sign::Pos => 1.0,
            Sign::Neg => 0.0,
        };
        QUAD_UV.map(|uv| T::compose(axis, sign, depth, uv))
    }
    pub fn normal<T>(self) -> T
    where
        T: Vec3Ext<f32>,
    {
        let sign = self.sign();
        let axis = self.axis();
        let value = match sign {
            Sign::Pos => 1.0,
            Sign::Neg => -1.0,
        };
        T::compose(axis, sign, value, [0.0, 0.0])
    }
    pub fn axis(self) -> Axis {
        match self {
            Side::XPos | Side::XNeg => Axis::X,
            Side::YPos | Side::YNeg => Axis::Y,
            Side::ZPos | Side::ZNeg => Axis::Z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Pos,
    Neg,
}

impl Axis {
    pub const fn swap(self, sign: Sign) -> AxisSwap {
        match (self, sign) {
            (Axis::X, Sign::Pos) => AxisSwap::XYZ,
            (Axis::X, Sign::Neg) => AxisSwap::XZY,
            (Axis::Y, Sign::Pos) => AxisSwap::YZX,
            (Axis::Y, Sign::Neg) => AxisSwap::YXZ,
            (Axis::Z, Sign::Pos) => AxisSwap::ZXY,
            (Axis::Z, Sign::Neg) => AxisSwap::ZYX,
        }
    }
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

pub trait Vec3Ext<T>: From<[T; 3]> + Into<[T; 3]> {
    fn split(self, axis: Axis, sign: Sign) -> (T, [T; 2]) {
        let [it, u, v] = self.into().axis_swap(axis.swap(sign));
        (it, [u, v])
    }
    fn compose(axis: Axis, sign: Sign, it: T, [u, v]: [T; 2]) -> Self {
        ([it, u, v].axis_swap(axis.swap(sign).inverse())).into()
    }
    fn axis_swap(self, swap: AxisSwap) -> Self {
        let [x, y, z] = self.into();
        match swap {
            AxisSwap::XYZ => [x, y, z],
            AxisSwap::XZY => [x, z, y],
            AxisSwap::YXZ => [y, x, z],
            AxisSwap::YZX => [y, z, x],
            AxisSwap::ZXY => [z, x, y],
            AxisSwap::ZYX => [z, y, x],
        }
        .into()
    }
    // TODO: remove
    fn zips(self, rhs: impl Vec3Ext<T>, mut map: impl FnMut(T, T) -> T) -> Self {
        let [x1, y1, z1] = self.into();
        let [x2, y2, z2] = rhs.into();
        [map(x1, x2), map(y1, y2), map(z1, z2)].into()
    }
}
impl<T> Vec3Ext<T> for [T; 3] {}
impl Vec3Ext<f32> for Vec3 {}
impl Vec3Ext<i32> for IVec3 {}

pub trait Vec2Ext<T>: From<[T; 2]> + Into<[T; 2]> {}
impl<T> Vec2Ext<T> for [T; 2] {}
impl Vec2Ext<f32> for Vec2 {}
impl Vec2Ext<i32> for IVec2 {}

impl Index<Axis> for IVec3 {
    type Output = i32;

    fn index(&self, index: Axis) -> &Self::Output {
        match index {
            Axis::X => self.index(0),
            Axis::Y => self.index(1),
            Axis::Z => self.index(2),
        }
    }
}
impl Index<Axis> for Vec3 {
    type Output = f32;

    fn index(&self, index: Axis) -> &Self::Output {
        match index {
            Axis::X => self.index(0),
            Axis::Y => self.index(1),
            Axis::Z => self.index(2),
        }
    }
}
impl IndexMut<Axis> for IVec3 {
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        match index {
            Axis::X => self.index_mut(0),
            Axis::Y => self.index_mut(1),
            Axis::Z => self.index_mut(2),
        }
    }
}
impl IndexMut<Axis> for Vec3 {
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        match index {
            Axis::X => self.index_mut(0),
            Axis::Y => self.index_mut(1),
            Axis::Z => self.index_mut(2),
        }
    }
}
