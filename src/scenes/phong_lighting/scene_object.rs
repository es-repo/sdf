use crate::geometry::{Sdf3d, Sphere, Vec3};
use crate::procedural::smooth_combine::smooth_union_many;
use pixels::wgpu::Color;

#[derive(Clone, Copy)]
pub struct SceneObject {
    position: Vec3,
    spheres: [Sphere; 5],
    rotation_y: f32,
    pub color: Color,
}

impl SceneObject {
    pub fn new(position: Vec3, core_radius: f32, color: Color) -> Self {
        let side_radius = core_radius * 0.8;
        let side_shift = core_radius * 1.2;

        let spheres = [
            Sphere {
                center: Vec3::new(0.0, 0.0, 0.0),
                radius: 1.0,
                color,
            },
            Sphere {
                center: Vec3::new(-side_shift, side_shift, 0.0),
                radius: side_radius,
                color,
            },
            Sphere {
                center: Vec3::new(-side_shift, -side_shift, 0.0),
                radius: side_radius,
                color,
            },
            Sphere {
                center: Vec3::new(side_shift, -side_shift, 0.0),
                radius: side_radius,
                color,
            },
            Sphere {
                center: Vec3::new(side_shift, side_shift, 0.0),
                radius: side_radius,
                color,
            },
        ];

        Self {
            position,
            spheres,
            rotation_y: 0.0,
            color,
        }
    }

    pub fn dist(&self, point: Vec3) -> f32 {
        let local_point = (point - self.position).rotate(Vec3::y_axis(), -self.rotation_y);
        let distances = self.spheres.map(|s| s.dist(local_point));
        smooth_union_many(distances, 0.5).unwrap()
    }

    pub fn rotate_y(&mut self, radians: f32) {
        self.rotation_y += radians;
    }
}
