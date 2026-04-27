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
