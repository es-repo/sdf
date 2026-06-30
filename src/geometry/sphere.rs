use crate::geometry::{SignedDistance3d, Vec3};
use egui_wgpu::wgpu::Color;

#[derive(Copy, Clone, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub color: Color,
}

impl SignedDistance3d for Sphere {
    fn dist(&self, point: Vec3) -> f32 {
        self.center.dist(point) - self.radius
    }
}
