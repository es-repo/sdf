use super::{Sdf, Vec2};
use pixels::wgpu::Color;

#[derive(Copy, Clone, Debug)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
    pub color: Color,
}

impl Circle {
    pub fn dist_squared(&self, v: &Vec2) -> f32 {
        self.center.dist_squared(v) - self.radius
    }

    pub fn dist_squared_radius_squared(&self, v: &Vec2) -> f32 {
        self.center.dist_squared(v) - self.radius * self.radius
    }
}

impl Sdf for Circle {
    fn dist(&self, v: &Vec2) -> f32 {
        self.center.dist(v) - self.radius
    }
}
