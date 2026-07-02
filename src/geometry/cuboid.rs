use crate::geometry::{Quat, SignedDistance3d, Vec3};
use pixels::wgpu::Color;

#[derive(Copy, Clone, Debug)]
pub struct Cuboid {
    pub center: Vec3,
    pub half_size: Vec3,
    pub color: Color,
    pub rotation: Quat,
}

impl SignedDistance3d for Cuboid {
    fn dist(&self, point: Vec3) -> f32 {
        let local_point = self.rotation.conjugate() * (point - self.center);
        let offset = local_point.abs() - self.half_size;

        let outside = offset.max(Vec3::zero()).len();
        let inside = offset.x.max(offset.y.max(offset.z)).min(0.0);

        outside + inside
    }
}
