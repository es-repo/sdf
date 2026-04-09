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

    fn hash(&self) -> Self {
        let x = self.dot(Self::new(127.1, 311.7, 74.7));
        let y = self.dot(Self::new(269.5, 183.3, 246.1));
        let z = self.dot(Self::new(113.5, 271.9, 124.6));
        let p = Self::new(x, y, z);
        (p.sin() * 43758.547).fract_glsl() * 2.0 - 1.0
    }

    // Adapted to 3D simplex coordinates, following the same gradient-noise pattern as Vec2.
    pub fn noise_simplex(&self) -> f32 {
        const F3: f32 = 1.0 / 3.0;
        const G3: f32 = 1.0 / 6.0;

        let s = (self.x + self.y + self.z) * F3;
        let i = (*self + s).floor();

        let t = (i.x + i.y + i.z) * G3;
        let x0 = *self - i + t;

        let rank = Self::new(
            step(x0.y, x0.x) + step(x0.z, x0.x),
            step(x0.x, x0.y) + step(x0.z, x0.y),
            step(x0.x, x0.z) + step(x0.y, x0.z),
        );

        let i1 = Self::new(step(1.5, rank.x), step(1.5, rank.y), step(1.5, rank.z));
        let i2 = Self::new(step(0.5, rank.x), step(0.5, rank.y), step(0.5, rank.z));

        let x1 = x0 - i1 + G3;
        let x2 = x0 - i2 + 2.0 * G3;
        let x3 = x0 - 1.0 + 3.0 * G3;

        let h0 = (0.6 - x0.dot(x0)).max(0.0);
        let h1 = (0.6 - x1.dot(x1)).max(0.0);
        let h2 = (0.6 - x2.dot(x2)).max(0.0);
        let h3 = (0.6 - x3.dot(x3)).max(0.0);

        let n0 = h0 * h0 * h0 * h0 * x0.dot(i.hash());
        let n1 = h1 * h1 * h1 * h1 * x1.dot((i + i1).hash());
        let n2 = h2 * h2 * h2 * h2 * x2.dot((i + i2).hash());
        let n3 = h3 * h3 * h3 * h3 * x3.dot((i + 1.0).hash());

        32.0 * (n0 + n1 + n2 + n3)
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

fn step(edge: f32, x: f32) -> f32 {
    if x < edge { 0.0 } else { 1.0 }
}
