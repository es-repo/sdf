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

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let axis_len = axis.len();

        if axis_len == 0.0 {
            return Self::identity();
        }

        let axis = axis / axis_len;
        let (sin, cos) = (angle * 0.5).sin_cos();

        Self {
            x: axis.x * sin,
            y: axis.y * sin,
            z: axis.z * sin,
            w: cos,
        }
    }

    pub fn normalize(self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();

        if len == 0.0 {
            return Self::identity();
        }

        Self {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
            w: self.w / len,
        }
    }
}

impl Default for Quat {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul<Vec3> for Quat {
    type Output = Vec3;

    fn mul(self, v: Vec3) -> Vec3 {
        let qv = Vec3::new(self.x, self.y, self.z);
        let t = qv.cross(v) * 2.0;
        v + t * self.w + qv.cross(t)
    }
}

impl Mul for Quat {
    type Output = Quat;

    fn mul(self, rhs: Quat) -> Self::Output {
        Self {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}
