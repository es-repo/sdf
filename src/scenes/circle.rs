use crate::Vec2;
use pixels::wgpu::Color;

pub struct Circle {
    pub center: Vec2<f64>,
    pub radius: f64,
    pub color: Color,
}

impl Circle {
    pub fn dist_squared(&self, other: &Vec2<f64>) -> f64 {
        self.center.dist_squared(other) - self.radius
    }

    pub fn dist_squared_radius_squared(&self, other: &Vec2<f64>) -> f64 {
        self.center.dist_squared(other) - self.radius * self.radius
    }

    pub fn dist(&self, other: &Vec2<f64>) -> f64 {
        self.center.dist(other) - self.radius
    }
}
