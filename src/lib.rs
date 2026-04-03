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

    fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    fn fract(self) -> Self {
        Self::new(self.x.fract(), self.y.fract())
    }

    fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

impl Add for Vec2<f64> {
    type Output = Vec2<f64>;

    fn add(self, rhs: Vec2<f64>) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
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

pub fn mix(a: f64, b: f64, t: f64) -> f64 {
    a * (1.0 - t) + b * t
}

pub fn grad(ix: i32, iy: i32) -> Vec2<f64> {
    let mut n = ix.wrapping_add(iy.wrapping_mul(11111));
    n = (n << 13) ^ n;
    n = n
        .wrapping_mul(n.wrapping_mul(n).wrapping_mul(15731).wrapping_add(789221))
        .wrapping_add(1376312589)
        >> 16;

    n &= 7;

    let gr = Vec2::new((n & 1) as f64, (n >> 1) as f64) * 2.0 - Vec2::new(1.0, 1.0);

    if n >= 6 {
        Vec2::new(0.0, gr.x)
    } else if n >= 4 {
        Vec2::new(gr.x, 0.0)
    } else {
        gr
    }
}

pub fn noise(p: Vec2<f64>) -> f64 {
    let i = p.floor();
    let f = p.fract();
    let u = f * f * (Vec2::new(3.0, 3.0) - f * 2.0);

    let ixi = i.x as i32;
    let iyi = i.y as i32;

    let a = grad(ixi, iyi).dot(f - Vec2::new(0.0, 0.0));
    let b = grad(ixi + 1, iyi).dot(f - Vec2::new(1.0, 0.0));
    let c = grad(ixi, iyi + 1).dot(f - Vec2::new(0.0, 1.0));
    let d = grad(ixi + 1, iyi + 1).dot(f - Vec2::new(1.0, 1.0));

    mix(mix(a, b, u.x), mix(c, d, u.x), u.y)
}

pub fn warp(p: Vec2<f64>, scale: f64, strength: f64) -> Vec2<f64> {
    let offset_x = noise(p * scale + Vec2::<f64>::new(0.0, 100.0));
    let offset_y = noise(p * scale + Vec2::<f64>::new(100.0, 0.0));

    p + Vec2::new(offset_x, offset_y) * strength
}

pub fn smooth_union(d1: f64, d2: f64, k: f64) -> (f64, f64) {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    let d = d2 * (1.0 - h) + d1 * h - k * h * (1.0 - h);
    (d, h)
}
