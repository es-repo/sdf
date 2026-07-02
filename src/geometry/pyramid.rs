use crate::geometry::{Quat, SignedDistance3d, Vec3};
use pixels::wgpu::Color;

#[derive(Copy, Clone, Debug)]
pub struct Pyramid {
    pub center: Vec3,
    pub height: f32,
    pub base_half_size: f32,
    pub color: Color,
    pub rotation: Quat,
}

impl SignedDistance3d for Pyramid {
    fn dist(&self, point: Vec3) -> f32 {
        let local_point = self.rotation.conjugate() * (point - self.center);
        let mut point = local_point;
        let base_half_size_squared = self.base_half_size * self.base_half_size;
        let slant_height_squared = self.height * self.height + base_half_size_squared;

        point.x = point.x.abs();
        point.z = point.z.abs();

        if point.z > point.x {
            std::mem::swap(&mut point.x, &mut point.z);
        }

        point.x -= self.base_half_size;
        point.z -= self.base_half_size;

        let q = Vec3::new(
            point.z,
            self.height * point.y - self.base_half_size * point.x,
            self.height * point.x + self.base_half_size * point.y,
        );

        let s = (-q.x).max(0.0);
        let t =
            ((q.y - self.base_half_size * point.z) / (slant_height_squared + base_half_size_squared)).clamp(0.0, 1.0);
        let a = slant_height_squared * (q.x + s) * (q.x + s) + q.y * q.y;
        let b = slant_height_squared * (q.x + self.base_half_size * t) * (q.x + self.base_half_size * t)
            + (q.y - slant_height_squared * t) * (q.y - slant_height_squared * t);

        let side_distance_squared = if q.y.min(-q.x * slant_height_squared - self.base_half_size * q.y) > 0.0 {
            0.0
        } else {
            a.min(b)
        };
        let side_distance = ((side_distance_squared + q.z * q.z) / slant_height_squared).sqrt();

        let base_offset = Vec3::new(
            (local_point.x.abs() - self.base_half_size).max(0.0),
            local_point.y,
            (local_point.z.abs() - self.base_half_size).max(0.0),
        );
        let distance = side_distance.min(base_offset.len());

        let slope = self.base_half_size / self.height;
        let inside = local_point.y >= 0.0
            && local_point.x.abs() + slope * local_point.y <= self.base_half_size
            && local_point.z.abs() + slope * local_point.y <= self.base_half_size;

        if inside { -distance } else { distance }
    }
}
