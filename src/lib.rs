pub mod scenes;

use pixels::wgpu::Color;
use std::ops::{Add, Mul, Sub};

pub trait ColorExt {
    fn to_u8_array(&self) -> [u8; 4];
    fn blend(&mut self, dst: Color) -> &mut Self;
}

impl ColorExt for Color {
    fn to_u8_array(&self) -> [u8; 4] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ]
    }

    fn blend(&mut self, dst: Color) -> &mut Self {
        let sa = self.a.clamp(0.0, 1.0);
        let da = dst.a.clamp(0.0, 1.0);

        let out_a = sa + da * (1.0 - sa);

        if out_a == 0.0 {
            self.r = 0.0;
            self.g = 0.0;
            self.b = 0.0;
            self.a = 0.0;
        } else {
            self.r = (self.r * sa + dst.r * da * (1.0 - sa)) / out_a;
            self.g = (self.g * sa + dst.g * da * (1.0 - sa)) / out_a;
            self.b = (self.b * sa + dst.b * da * (1.0 - sa)) / out_a;
            self.a = out_a;
        }

        self
    }
}

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
    pub fn to_aspect_ndc(&self, w: u32, h: u32) -> Vec2<f64> {
        let xf = self.x as f64 + 0.5;
        let yf = self.y as f64 + 0.5;

        let nx = (2.0 * xf - w as f64) / h as f64;
        let ny = (h as f64 - 2.0 * yf) / h as f64;

        Vec2 { x: nx, y: ny }
    }
}

impl Vec2<f64> {
    pub fn len(&self) -> f64 {
        self.len_squared().sqrt()
    }

    pub fn len_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    pub fn dist_squared(&self, other: &Vec2<f64>) -> f64 {
        (self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y)
    }

    pub fn dist(&self, other: &Vec2<f64>) -> f64 {
        self.dist_squared(other).sqrt()
    }

    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    /*pub fn fract(self) -> Self {
        Self::new(self.x.fract(), self.y.fract())
    }*/

    pub fn dot(self, other: Self) -> f64 {
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
    fn fract_glsl(self) -> Self {
        Self {
            x: self.x - self.x.floor(),
            y: self.y - self.y.floor(),
        }
    }

    fn hash(&self) -> Self {
        let x = self.dot(Self { x: 127.1, y: 311.7 });
        let y = self.dot(Self { x: 269.5, y: 183.3 });
        let p = Self { x, y };
        (p.sin() * 43758.5453123).fract_glsl() * 2.0 - 1.0
    }

    // Adopted from https://www.shadertoy.com/view/Msf3WH
    pub fn noise_simplex(&self) -> f64 {
        const K1: f64 = 0.366025404; // (sqrt(3)-1)/2;
        const K2: f64 = 0.211324865; // (3-sqrt(3))/6;

        let i = (*self + (self.x + self.y) * K1).floor();
        let a = *self - i + (i.x + i.y) * K2;

        let m = step(a.y, a.x);
        let o = Vec2::new(m, 1.0 - m);
        let b = a - o + K2;
        let c = a - 1.0 + 2.0 * K2;

        let ha = (0.5 - a.dot(a)).max(0.0);
        let hb = (0.5 - b.dot(b)).max(0.0);
        let hc = (0.5 - c.dot(c)).max(0.0);

        let na = ha * ha * ha * ha * a.dot(i.hash());
        let nb = hb * hb * hb * hb * b.dot((i + o).hash());
        let nc = hc * hc * hc * hc * c.dot((i + 1.0).hash());

        70.0 * (na + nb + nc)
    }

    pub fn fbm(
        &self,
        octaves: u32,
        amplitude: f64,
        gain: f64,
        lacunarity: f64,
        noise: impl Fn(Vec2<f64>) -> f64,
    ) -> f64 {
        self.fbm_with_transform(octaves, amplitude, gain, noise, |coord| coord * lacunarity)
    }

    pub fn fbm_with_transform(
        &self,
        octaves: u32,
        amplitude: f64,
        gain: f64,
        noise: impl Fn(Vec2<f64>) -> f64,
        transform: impl Fn(Vec2<f64>) -> Vec2<f64>,
    ) -> f64 {
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
}

impl Add for Vec2<f64> {
    type Output = Vec2<f64>;

    fn add(self, rhs: Vec2<f64>) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<f64> for Vec2<f64> {
    type Output = Vec2<f64>;

    fn add(self, rhs: f64) -> Self::Output {
        Vec2::new(self.x + rhs, self.y + rhs)
    }
}

impl Sub<f64> for Vec2<f64> {
    type Output = Vec2<f64>;

    fn sub(self, rhs: f64) -> Self::Output {
        Vec2::new(self.x - rhs, self.y - rhs)
    }
}

impl Sub for Vec2<f64> {
    type Output = Vec2<f64>;

    fn sub(self, rhs: Vec2<f64>) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f64> for Vec2<f64> {
    type Output = Vec2<f64>;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vec2<f64>> for Vec2<f64> {
    type Output = Vec2<f64>;

    fn mul(self, rhs: Vec2<f64>) -> Self::Output {
        Vec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

pub fn smooth_union(d1: f64, d2: f64, k: f64) -> (f64, f64) {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    let d = d2 * (1.0 - h) + d1 * h - k * h * (1.0 - h);
    (d, h)
}

fn step(edge: f64, x: f64) -> f64 {
    if x < edge { 0.0 } else { 1.0 }
}
