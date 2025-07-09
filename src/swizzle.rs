use std::ops::Mul;

#[derive(Debug, Clone, Copy)]
pub enum Swizzle3 {
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZXY,
    ZYX,
}

impl Mul<[f32; 3]> for Swizzle3 {
    type Output = [f32; 3];

    fn mul(self, [x, y, z]: [f32; 3]) -> Self::Output {
        match self {
            Swizzle3::XYZ => [x, y, z],
            Swizzle3::XZY => [x, z, y],
            Swizzle3::YXZ => [y, x, z],
            Swizzle3::YZX => [y, z, x],
            Swizzle3::ZXY => [z, x, y],
            Swizzle3::ZYX => [z, y, x],
        }
    }
}
