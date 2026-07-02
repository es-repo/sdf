use crate::color_ext::ColorExt;
use crate::geometry::{Plane, SignedDistance3d, Vec3};
use crate::rendering::SdfSample;
use pixels::wgpu::Color;

#[derive(Copy, Clone, Debug)]
pub(super) struct Ground {
    plane: Plane,
    size: f32,
}

impl Ground {
    pub fn new(color: Color, offset: f32, size: f32) -> Self {
        Self {
            plane: Plane {
                normal: Vec3::y_axis(),
                offset,
                color,
            },
            size,
        }
    }

    pub fn dist(&self, point: Vec3, scene_time: f32, animated: bool) -> f32 {
        let dist = self.plane.dist(point);

        let dist = dist
            .max(point.x - self.size)
            .max(-point.x - self.size)
            .max(point.z - self.size)
            .max(-point.z - self.size)
            .max(-point.y - 1.0);

        if !animated {
            return dist;
        }

        let s = 0.2;
        let speed = 2.0;
        let dist = dist + s * (point.x + scene_time * speed).sin() + s * (point.z + scene_time * speed).sin();
        dist
    }

    pub fn sample(&self, point: Vec3, scene_time: f32, animated: bool) -> SdfSample {
        let size = 1.5;
        let x_even = (point.x * size).floor() as i32 % 2 == 0;
        let z_even = (point.z * size).floor() as i32 % 2 == 0;

        let color = if (x_even && !z_even) || (!x_even && z_even) {
            self.plane.color.lerp(Color::WHITE, 0.8)
        } else {
            self.plane.color
        };

        SdfSample::new(self.dist(point, scene_time, animated), color)
    }
}
