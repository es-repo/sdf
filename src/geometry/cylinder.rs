use crate::geometry::{Quat, SignedDistance3d, Vec2, Vec3};
use pixels::wgpu::Color;

#[derive(Copy, Clone, Debug)]
pub struct Cylinder {
    pub center: Vec3,
    pub radius: f32,
    pub half_height: f32,
    pub color: Color,
    pub rotation: Quat,
}

impl SignedDistance3d for Cylinder {
    fn dist(&self, point: Vec3) -> f32 {
        let local_point = self.rotation.conjugate() * (point - self.center);
        let radial_distance = Vec2::new(local_point.x, local_point.z).len() - self.radius;
        let vertical_distance = local_point.y.abs() - self.half_height;

        let outside = Vec2::new(radial_distance.max(0.0), vertical_distance.max(0.0)).len();
        let inside = radial_distance.max(vertical_distance).min(0.0);

        outside + inside
    }
}
