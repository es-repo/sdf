use super::Vec2;
use pixels::wgpu::Color;

pub struct Circle {
    pub center: Vec2<f32>,
    pub radius: f32,
    pub color: Color,
}

impl Circle {
    #[allow(dead_code)]
    pub fn dist_squared(&self, other: &Vec2<f32>) -> f32 {
        self.center.dist_squared(other) - self.radius
    }

    pub fn dist_squared_radius_squared(&self, other: &Vec2<f32>) -> f32 {
        self.center.dist_squared(other) - self.radius * self.radius
    }

    pub fn dist(&self, other: &Vec2<f32>) -> f32 {
        self.center.dist(other) - self.radius
    }
}
