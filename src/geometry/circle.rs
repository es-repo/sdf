use super::{SignedDistance2d, Vec2};
use pixels::wgpu::Color;

#[derive(Copy, Clone, Debug)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
    pub color: Color,
}

impl Circle {
    pub fn dist_squared(&self, point: Vec2) -> f32 {
        self.center.dist_squared(point) - self.radius
    }

    pub fn dist_squared_radius_squared(&self, point: Vec2) -> f32 {
        self.center.dist_squared(point) - self.radius * self.radius
    }
}

impl SignedDistance2d for Circle {
    fn dist(&self, point: Vec2) -> f32 {
        self.center.dist(point) - self.radius
    }
}
