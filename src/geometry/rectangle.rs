use crate::geometry::{Sdf, Vec2};
use pixels::wgpu::Color;

pub struct Rectangle {
    pub center: Vec2<f32>,
    pub vertex: Vec2<f32>,
    pub color: Color,
    pub rotation: f32,
}

impl Sdf for Rectangle {
    fn dist(&self, v: &Vec2<f32>) -> f32 {
        let p = (*v - self.center).rotate(-self.rotation);
        let d = p.abs() - self.vertex;

        let outside = Vec2::new(d.x.max(0.0), d.y.max(0.0)).len();
        let inside = d.x.max(d.y).min(0.0);

        outside + inside
    }
}
