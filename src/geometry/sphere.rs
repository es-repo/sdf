use crate::geometry::{Sdf3d, Vec3};
use egui_wgpu::wgpu::Color;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub color: Color,
}

impl Sdf3d for Sphere {
    fn dist(&self, v: &Vec3) -> f32 {
        self.center.dist(v) - self.radius
    }
}
