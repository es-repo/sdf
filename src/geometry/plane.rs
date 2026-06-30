use crate::geometry::{SignedDistance3d, Vec3};
use egui_wgpu::wgpu::Color;

#[derive(Copy, Clone, Debug)]
pub struct Plane {
    pub normal: Vec3,
    pub offset: f32,
    pub color: Color,
}

impl SignedDistance3d for Plane {
    fn dist(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.offset
    }
}
