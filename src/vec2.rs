use crate::{fast_floor, perm};
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, PartialOrd, Eq, PartialEq, Debug)]
pub struct Vec2<T: PartialOrd + PartialEq + Clone + Copy> {
    pub x: T,
    pub y: T,
}

impl<T: PartialOrd + PartialEq + Clone + Copy> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Vec2<u32> {
    pub fn to_aspect_ndc(&self, w: u32, h: u32) -> Vec2<f32> {
        let xf = self.x as f32 + 0.5;
        let yf = self.y as f32 + 0.5;

        let nx = (2.0 * xf - w as f32) / h as f32;
        let ny = (h as f32 - 2.0 * yf) / h as f32;

        Vec2 { x: nx, y: ny }
    }
}

impl Vec2<f32> {
    const GRAD2: [[f32; 2]; 8] = [
        [1.0, 0.0],
        [-1.0, 0.0],
        [0.0, 1.0],
        [0.0, -1.0],
        [1.0, 1.0],
        [-1.0, 1.0],
        [1.0, -1.0],
        [-1.0, -1.0],
    ];

    pub fn len(&self) -> f32 {
        self.len_squared().sqrt()
    }

    pub fn len_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn dist_squared(&self, other: &Vec2<f32>) -> f32 {
        (self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y)
    }

    pub fn dist(&self, other: &Vec2<f32>) -> f32 {
        self.dist_squared(other).sqrt()
    }

    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    /*pub fn fract(self) -> Self {
        Self::new(self.x.fract(), self.y.fract())
    }*/

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn sin(&self) -> Self {
        Self {
            x: self.x.sin(),
            y: self.y.sin(),
        }
    }

    pub fn cos(&self) -> Self {
        Self {
            x: self.x.cos(),
            y: self.y.cos(),
        }
    }

    // Version of `fract` that corresponds to GLSL's `fract` function,
    // where, for example fract_glsl(-1.2) = 0.8
    pub fn fract_glsl(self) -> Self {
        Self {
            x: self.x - self.x.floor(),
            y: self.y - self.y.floor(),
        }
    }

    // Canonical 2D simplex noise with a fixed gradient set and permutation hashing.
    pub fn noise_simplex(&self) -> f32 {
        const F2: f32 = 0.3660254; // (sqrt(3)-1)/2
        const G2: f32 = 0.21132487; // (3-sqrt(3))/6

        let s = (self.x + self.y) * F2;
        let i = fast_floor(self.x + s);
        let j = fast_floor(self.y + s);

        let t = (i + j) as f32 * G2;
        let x0 = self.x - (i as f32 - t);
        let y0 = self.y - (j as f32 - t);

        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };

        let x1 = x0 - i1 as f32 + G2;
        let y1 = y0 - j1 as f32 + G2;
        let x2 = x0 - 1.0 + 2.0 * G2;
        let y2 = y0 - 1.0 + 2.0 * G2;

        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;

        let gi0 = perm(ii + perm(jj) as usize) % 8;
        let gi1 = perm(ii + i1 as usize + perm(jj + j1 as usize) as usize) % 8;
        let gi2 = perm(ii + 1 + perm(jj + 1) as usize) % 8;

        let n0 = Self::corner_contrib_2d(gi0 as usize, x0, y0);
        let n1 = Self::corner_contrib_2d(gi1 as usize, x1, y1);
        let n2 = Self::corner_contrib_2d(gi2 as usize, x2, y2);

        70.0 * (n0 + n1 + n2)
    }

    pub fn fbm(
        &self,
        octaves: u32,
        amplitude: f32,
        gain: f32,
        lacunarity: f32,
        noise: impl Fn(Vec2<f32>) -> f32,
    ) -> f32 {
        self.fbm_with_transform(octaves, amplitude, gain, noise, |coord| coord * lacunarity)
    }

    pub fn fbm_with_transform(
        &self,
        octaves: u32,
        amplitude: f32,
        gain: f32,
        noise: impl Fn(Vec2<f32>) -> f32,
        transform: impl Fn(Vec2<f32>) -> Vec2<f32>,
    ) -> f32 {
        let mut coord = *self;
        let mut value = 0.0;
        let mut amplitude = amplitude;

        for _ in 0..octaves {
            value += amplitude * noise(coord);
            coord = transform(coord);
            amplitude *= gain;
        }

        value
    }

    pub fn fbm_rotated(&self, octaves: u32, amplitude: f32, gain: f32) -> f32 {
        self.fbm_with_transform(
            octaves,
            amplitude,
            gain,
            |coord| coord.noise_simplex(),
            |coord| Vec2::new(1.6 * coord.x + 1.2 * coord.y, -1.2 * coord.x + 1.6 * coord.y),
        )
    }

    fn corner_contrib_2d(grad_index: usize, x: f32, y: f32) -> f32 {
        let t = 0.5 - x * x - y * y;
        if t <= 0.0 {
            return 0.0;
        }

        let grad = Self::GRAD2[grad_index];
        let t2 = t * t;
        let t4 = t2 * t2;
        t4 * (grad[0] * x + grad[1] * y)
    }
}

impl Add for Vec2<f32> {
    type Output = Vec2<f32>;

    fn add(self, rhs: Vec2<f32>) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<f32> for Vec2<f32> {
    type Output = Vec2<f32>;

    fn add(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x + rhs, self.y + rhs)
    }
}

impl Sub<f32> for Vec2<f32> {
    type Output = Vec2<f32>;

    fn sub(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x - rhs, self.y - rhs)
    }
}

impl Sub for Vec2<f32> {
    type Output = Vec2<f32>;

    fn sub(self, rhs: Vec2<f32>) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2<f32> {
    type Output = Vec2<f32>;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vec2<f32>> for Vec2<f32> {
    type Output = Vec2<f32>;

    fn mul(self, rhs: Vec2<f32>) -> Self::Output {
        Vec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}
