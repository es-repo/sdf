use crate::geometry::{SignedDistance3d, Sphere, Vec3};
use crate::procedural::NoiseSimplex;
use crate::procedural::smooth_combine::{smooth_subtraction_many, smooth_union_many};
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
    pub fn new(position: Vec3, core_radius: f32, color: Color, spheres_per_ring: usize) -> Self {
        let core_sphere = Sphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: core_radius,
            color,
        };

        let outer_spheres = Self::create_outer_spheres(core_radius, color, spheres_per_ring);

        Self {
            position,
            core_sphere,
            outer_spheres,
            is_out_phase: true,
            rotation_y: 0.0,
            color,
        }
    }

    fn create_outer_spheres(core_radius: f32, color: Color, spheres_per_ring: usize) -> Vec<Sphere> {
        let outer_sphere_radius = core_radius * 0.3;
        let angle_step = std::f32::consts::TAU / spheres_per_ring as f32;
        let out_init_dist = core_radius * 1.1;

        let mut outer_spheres = Vec::with_capacity(spheres_per_ring * 3);
        for i in 0..spheres_per_ring {
            let angle = i as f32 * angle_step;
            let (sin, cos) = angle.sin_cos();

            let sphere = Sphere {
                center: Vec3::new(out_init_dist * cos, out_init_dist * sin, 0.0),
                radius: outer_sphere_radius,
                color,
            };
            outer_spheres.push(sphere);

            let sphere = Sphere {
                center: Vec3::new(0.0, out_init_dist * cos, out_init_dist * sin),
                radius: outer_sphere_radius,
                color,
            };
            outer_spheres.push(sphere);

            let sphere = Sphere {
                center: Vec3::new(out_init_dist * cos, 0.0, out_init_dist * sin),
                radius: outer_sphere_radius,
                color,
            };
            outer_spheres.push(sphere);
        }

        outer_spheres
    }

    pub fn set_spheres_per_ring(&mut self, spheres_per_ring: usize) {
        if self.outer_spheres.len() != spheres_per_ring * 3 {
            self.outer_spheres = Self::create_outer_spheres(self.core_sphere.radius, self.color, spheres_per_ring);
        }
    }

    pub fn update(&mut self, scene_time: f32) {
        self.rotation_y += scene_time * 0.1;
        let phase = scene_time.sin();
        self.is_out_phase = phase >= 0.0;

        let position_scale = 0.5 + 0.45 * phase.abs();

        for sphere in &mut self.outer_spheres {
            sphere.center = sphere.center * position_scale;
        }
    }

    pub fn dist(&self, point: Vec3) -> f32 {
        let local_point = (point - self.position).rotate(Vec3::y_axis(), -self.rotation_y);
        let spheres = std::iter::once(&self.core_sphere).chain(&self.outer_spheres);

        let distances = spheres.map(|s| s.dist(local_point));
        let dist = if self.is_out_phase {
            smooth_union_many(distances, 0.5).unwrap()
        } else {
            smooth_subtraction_many(distances, 0.5).unwrap()
        };

        let noise_strength = 10.5;

        let core_sphere_dist = self.core_sphere.dist(local_point);
        let k = 1.0;
        let h = 1.0 - ((core_sphere_dist - dist) * k).max(0.0).clamp(0.0, 1.0);

        let dist = dist + 0.01 * h * (local_point * noise_strength).noise_simplex();
        dist
    }
}
