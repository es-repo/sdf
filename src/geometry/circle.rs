use super::{Sdf, Vec2};
use pixels::wgpu::Color;

pub struct Circle {
    pub center: Vec2<f32>,
    pub radius: f32,
    pub color: Color,
}

impl Circle {
    #[allow(dead_code)]
    pub fn dist_squared(&self, v: &Vec2<f32>) -> f32 {
        self.center.dist_squared(v) - self.radius
    }

    pub fn dist_squared_radius_squared(&self, v: &Vec2<f32>) -> f32 {
        self.center.dist_squared(v) - self.radius * self.radius
    }
}

impl Sdf for Circle {
    fn dist(&self, v: &Vec2<f32>) -> f32 {
        self.center.dist(v) - self.radius
    }
}
