use std::ops::{Add, Div, Mul, Sub};

#[derive(Clone, Copy, PartialOrd, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn len(&self) -> f32 {
        self.len_squared().sqrt()
    }

    pub fn len_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn normalize(self) -> Self {
        self / self.len()
    }

    pub fn dist_squared(&self, other: &Vec2) -> f32 {
        (self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y)
    }

    pub fn dist(&self, other: &Vec2) -> f32 {
        self.dist_squared(other).sqrt()
    }

    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    /*pub fn fract(self) -> Self {
        Self::new(self.x.fract(), self.y.fract())
    }*/

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Reflects this vector around a normalized surface normal.
    pub fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }

    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }

    pub fn rotate(self, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(self.x * cos - self.y * sin, self.x * sin + self.y * cos)
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

    pub fn exp(&self) -> Self {
        Self {
            x: self.x.exp(),
            y: self.y.exp(),
        }
    }

    pub fn round(&self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
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

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<f32> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x + rhs, self.y + rhs)
    }
}

impl Sub<f32> for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x - rhs, self.y - rhs)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vec2> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x / rhs, self.y / rhs)
    }
}
