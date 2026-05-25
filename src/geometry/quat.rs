use crate::geometry::Vec3;
use std::ops::Mul;

#[derive(Clone, Copy, PartialOrd, PartialEq, Debug)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

impl Default for Quat {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul<Vec3<f32>> for Quat {
    type Output = Vec3<f32>;

    fn mul(self, v: Vec3<f32>) -> Vec3<f32> {
        let qv = Vec3::new(self.x, self.y, self.z);
        let t = qv.cross(v) * 2.0;
        v + t * self.w + qv.cross(t)
    }
}
