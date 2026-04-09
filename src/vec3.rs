use crate::{Vec2, fast_floor, perm};
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
    const GRAD3: [[f32; 3]; 12] = [
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [1.0, -1.0, 0.0],
        [-1.0, -1.0, 0.0],
        [1.0, 0.0, 1.0],
        [-1.0, 0.0, 1.0],
        [1.0, 0.0, -1.0],
        [-1.0, 0.0, -1.0],
        [0.0, 1.0, 1.0],
        [0.0, -1.0, 1.0],
        [0.0, 1.0, -1.0],
        [0.0, -1.0, -1.0],
    ];

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

    // Canonical 3D simplex noise with a fixed gradient set and permutation hashing.
    pub fn noise_simplex(&self) -> f32 {
        const F3: f32 = 1.0 / 3.0;
        const G3: f32 = 1.0 / 6.0;

        let s = (self.x + self.y + self.z) * F3;
        let i = fast_floor(self.x + s);
        let j = fast_floor(self.y + s);
        let k = fast_floor(self.z + s);

        let t = (i + j + k) as f32 * G3;
        let x0 = self.x - (i as f32 - t);
        let y0 = self.y - (j as f32 - t);
        let z0 = self.z - (k as f32 - t);

        let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
            if y0 >= z0 {
                (1, 0, 0, 1, 1, 0)
            } else if x0 >= z0 {
                (1, 0, 0, 1, 0, 1)
            } else {
                (0, 0, 1, 1, 0, 1)
            }
        } else if y0 < z0 {
            (0, 0, 1, 0, 1, 1)
        } else if x0 < z0 {
            (0, 1, 0, 0, 1, 1)
        } else {
            (0, 1, 0, 1, 1, 0)
        };

        let x1 = x0 - i1 as f32 + G3;
        let y1 = y0 - j1 as f32 + G3;
        let z1 = z0 - k1 as f32 + G3;
        let x2 = x0 - i2 as f32 + 2.0 * G3;
        let y2 = y0 - j2 as f32 + 2.0 * G3;
        let z2 = z0 - k2 as f32 + 2.0 * G3;
        let x3 = x0 - 1.0 + 3.0 * G3;
        let y3 = y0 - 1.0 + 3.0 * G3;
        let z3 = z0 - 1.0 + 3.0 * G3;

        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;
        let kk = (k & 255) as usize;

        let gi0 = perm(ii + perm(jj + perm(kk) as usize) as usize) % 12;
        let gi1 = perm(ii + i1 as usize + perm(jj + j1 as usize + perm(kk + k1 as usize) as usize) as usize) % 12;
        let gi2 = perm(ii + i2 as usize + perm(jj + j2 as usize + perm(kk + k2 as usize) as usize) as usize) % 12;
        let gi3 = perm(ii + 1 + perm(jj + 1 + perm(kk + 1) as usize) as usize) % 12;

        let n0 = Self::corner_contrib_3d(gi0 as usize, x0, y0, z0);
        let n1 = Self::corner_contrib_3d(gi1 as usize, x1, y1, z1);
        let n2 = Self::corner_contrib_3d(gi2 as usize, x2, y2, z2);
        let n3 = Self::corner_contrib_3d(gi3 as usize, x3, y3, z3);

        32.0 * (n0 + n1 + n2 + n3)
    }

    fn corner_contrib_3d(grad_index: usize, x: f32, y: f32, z: f32) -> f32 {
        let t = 0.6 - x * x - y * y - z * z;
        if t <= 0.0 {
            return 0.0;
        }

        let grad = Self::GRAD3[grad_index];
        let t2 = t * t;
        let t4 = t2 * t2;
        t4 * (grad[0] * x + grad[1] * y + grad[2] * z)
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
