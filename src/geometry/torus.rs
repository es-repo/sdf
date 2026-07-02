use crate::geometry::{Quat, SignedDistance3d, Vec2, Vec3};
use pixels::wgpu::Color;

#[derive(Copy, Clone, Debug)]
pub struct Torus {
    pub center: Vec3,
    pub major_radius: f32,
    pub minor_radius: f32,
    pub color: Color,
    pub rotation: Quat,
}

impl SignedDistance3d for Torus {
    fn dist(&self, point: Vec3) -> f32 {
        let local_point = self.rotation.conjugate() * (point - self.center);
        let radial_distance = Vec2::new(local_point.x, local_point.z).len() - self.major_radius;
        Vec2::new(radial_distance, local_point.y).len() - self.minor_radius
    }
}
