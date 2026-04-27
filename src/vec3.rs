use crate::Vec2;
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, PartialOrd, Eq, PartialEq, Debug)]
pub struct Vec3<T: PartialOrd + PartialEq + Clone + Copy> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: PartialOrd + PartialEq + Clone + Copy> Vec3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl Vec3<f32> {
    pub fn from_2d(v2d: Vec2<f32>, z: f32) -> Self {
        Self { x: v2d.x, y: v2d.y, z }
    }

    pub fn len(&self) -> f32 {
        self.len_squared().sqrt()
    }

    pub fn len_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn dist_squared(&self, other: &Vec3<f32>) -> f32 {
        (self.x - other.x) * (self.x - other.x)
            + (self.y - other.y) * (self.y - other.y)
            + (self.z - other.z) * (self.z - other.z)
    }

    pub fn dist(&self, other: &Vec3<f32>) -> f32 {
        self.dist_squared(other).sqrt()
    }

    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor(), self.z.floor())
    }

    /*pub fn fract(self) -> Self {
        Self::new(self.x.fract(), self.y.fract(), self.z.fract())
    }*/

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn sin(&self) -> Self {
        Self {
            x: self.x.sin(),
            y: self.y.sin(),
            z: self.z.sin(),
        }
    }

    pub fn cos(&self) -> Self {
        Self {
            x: self.x.cos(),
            y: self.y.cos(),
            z: self.z.cos(),
        }
    }

    // Version of `fract` that corresponds to GLSL's `fract` function.
    pub fn fract_glsl(self) -> Self {
        Self {
            x: self.x - self.x.floor(),
            y: self.y - self.y.floor(),
            z: self.z - self.z.floor(),
        }
    }
}

impl Add for Vec3<f32> {
    type Output = Vec3<f32>;

    fn add(self, rhs: Vec3<f32>) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Add<f32> for Vec3<f32> {
    type Output = Vec3<f32>;

    fn add(self, rhs: f32) -> Self::Output {
        Vec3::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl Sub<f32> for Vec3<f32> {
    type Output = Vec3<f32>;

    fn sub(self, rhs: f32) -> Self::Output {
        Vec3::new(self.x - rhs, self.y - rhs, self.z - rhs)
    }
}

impl Sub for Vec3<f32> {
    type Output = Vec3<f32>;

    fn sub(self, rhs: Vec3<f32>) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3<f32> {
    type Output = Vec3<f32>;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<Vec3<f32>> for Vec3<f32> {
    type Output = Vec3<f32>;

    fn mul(self, rhs: Vec3<f32>) -> Self::Output {
        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}
