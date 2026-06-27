use crate::geometry::{Sdf3d, Sphere, Vec3};
use crate::procedural::smooth_combine::{smooth_subtraction_many, smooth_union_many};
use crate::procedural::{NoiseSimplex, smooth_subtraction_color};
use pixels::wgpu::Color;

#[derive(Clone)]
pub struct SceneObject {
    position: Vec3,
    pub core_sphere: Sphere,
    outer_spheres: Vec<Sphere>,
    is_out_phase: bool,
    rotation_y: f32,
    pub color: Color,
}

impl SceneObject {
    pub fn new(position: Vec3, core_radius: f32, color: Color) -> Self {
        let core_sphere = Sphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: core_radius,
            color,
        };

        let outer_sphere_radius = core_radius * 0.3;
        let n = 8;
        let ang_step = std::f32::consts::PI * 2.0 / n as f32;
        let out_init_dist = core_radius * 1.1;

        let mut outer_spheres = Vec::new();
        for i in 0..n {
            let ang = i as f32 * ang_step;

            let sphere = Sphere {
                center: Vec3::new(out_init_dist * ang.cos(), out_init_dist * ang.sin(), 0.0),
                radius: outer_sphere_radius,
                color,
            };
            outer_spheres.push(sphere);

            let sphere = Sphere {
                center: Vec3::new(0.0, out_init_dist * ang.cos(), out_init_dist * ang.sin()),
                radius: outer_sphere_radius,
                color,
            };
            outer_spheres.push(sphere);

            let sphere = Sphere {
                center: Vec3::new(out_init_dist * ang.cos(), 0.0, out_init_dist * ang.sin()),
                radius: outer_sphere_radius,
                color,
            };
            outer_spheres.push(sphere);
        }

        Self {
            position,
            core_sphere,
            outer_spheres,
            is_out_phase: true,
            rotation_y: 0.0,
            color,
        }
    }

    pub fn update(&mut self, scene_time: f32) {
        self.rotation_y += scene_time * 0.1;
        let phase = scene_time.sin();
        self.is_out_phase = phase >= 0.0;

        let position_scale = 0.5 + 0.5 * phase.abs();

        for i in (0..self.outer_spheres.len()) {
            self.outer_spheres[i].center = self.outer_spheres[i].center * position_scale;
        }
    }

    pub fn dist(&self, point: &Vec3) -> f32 {
        let local_point = (*point - self.position).rotate(Vec3::y_axis(), -self.rotation_y);
        let spheres = std::iter::once(&self.core_sphere).chain(&self.outer_spheres);

        let distances = spheres.map(|s| s.dist(&local_point));
        let dist = if self.is_out_phase {
            smooth_union_many(distances, 0.5).unwrap()
        } else {
            smooth_subtraction_many(distances, 0.5).unwrap()
        };

        let noise_strength = 10.5;

        let core_sphere_dist = self.core_sphere.dist(&local_point);
        let k = 0.8;
        let h = 1.0 - ((core_sphere_dist - dist) * k).max(0.0).clamp(0.0, 1.0);

        let dist = dist + 0.01 * h * (local_point * noise_strength).noise_simplex();
        dist
    }
}
