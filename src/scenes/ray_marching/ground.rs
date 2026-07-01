use crate::color_ext::ColorExt;
use crate::geometry::{Plane, SignedDistance3d, Vec3};
use crate::procedural::NoiseSimplex;
use crate::rendering::SdfSample;
use pixels::wgpu::Color;

#[derive(Copy, Clone, Debug)]
pub(super) struct Ground {
    plane: Plane,
}

impl Ground {
    pub fn new(color: Color, offset: f32) -> Self {
        Self {
            plane: Plane {
                normal: Vec3::y_axis(),
                offset,
                color,
            },
        }
    }

    pub fn dist(&self, point: Vec3) -> f32 {
        let dist = self.plane.dist(point);
        let dist = dist + 0.01 * (point * 2.0).noise_simplex();
        dist
    }

    pub fn sample(&self, point: Vec3) -> SdfSample {
        let size = 0.5;
        let x_even = (point.x * size).floor() as i32 % 2 == 0;
        let z_even = (point.z * size).floor() as i32 % 2 == 0;

        let color = if (x_even && !z_even) || (!x_even && z_even) {
            self.plane.color.lerp(Color::WHITE, 0.5)
        } else {
            self.plane.color
        };

        SdfSample::new(self.dist(point), color)
    }
}
