use crate::color_ext::ColorExt;
use crate::geometry::{Sdf3d, Sphere, Vec2, Vec3};
use crate::input::InputState;
use crate::procedural::smooth_union::smooth_union_color;
use crate::rendering::{Camera, CameraController, CameraFrame};
use crate::scenes::{Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct RayMarching {
    camera: Camera,
    camera_controller: CameraController,
    max_distance: f32,
    hit_epsilon: f32,
    max_steps: usize,
}

impl Default for RayMarching {
    fn default() -> Self {
        let fov_y = 60f32.to_radians();
        Self {
            camera: Camera::new(fov_y, 1.0),
            camera_controller: CameraController::new(2.0),
            max_distance: 100.0,
            hit_epsilon: 0.0001,
            max_steps: 100,
        }
    }
}

struct RayMarchingFrame {
    camera_frame: CameraFrame,
    sphere_1: Sphere,
    sphere_2: Sphere,
    //sphere_3: Sphere,
    max_distance: f32,
    hit_epsilon: f32,
    max_steps: usize,
}

struct SdfSample {
    dist: f32,
    color: Color,
}

impl SdfSample {
    fn new(dist: f32, color: Color) -> Self {
        Self { dist, color }
    }
}

impl RayMarchingFrame {
    fn sample_scene(&self, point: Vec3) -> SdfSample {
        let sphere_1_dist = self.sphere_1.dist(&point);
        let sphere_2_dist = self.sphere_2.dist(&point);

        let dist = sphere_1_dist.max(sphere_2_dist);
        let (dist, color) = smooth_union_color(
            sphere_1_dist,
            self.sphere_1.color,
            sphere_2_dist,
            self.sphere_2.color,
            0.5,
        );

        SdfSample::new(dist, color)

        /*if sphere_1_dist < sphere_2_dist {
            SdfSample::new(sphere_1_dist, self.sphere_1.color)
        } else {
            SdfSample::new(sphere_2_dist, self.sphere_2.color)
        }*/
    }

    fn estimate_normal_tetrahedral(&self, p: Vec3) -> Vec3 {
        let e = 0.001;

        let k1 = Vec3::new(1.0, -1.0, -1.0);
        let k2 = Vec3::new(-1.0, -1.0, 1.0);
        let k3 = Vec3::new(-1.0, 1.0, -1.0);
        let k4 = Vec3::new(1.0, 1.0, 1.0);

        (k1 * self.sample_scene(p + k1 * e).dist
            + k2 * self.sample_scene(p + k2 * e).dist
            + k3 * self.sample_scene(p + k3 * e).dist
            + k4 * self.sample_scene(p + k4 * e).dist)
            .normalize()
    }

    fn estimate_normal_central_differences(&self, p: Vec3) -> Vec3 {
        let e = 0.001;

        Vec3::new(
            self.sample_scene(p + Vec3::new(e, 0.0, 0.0)).dist - self.sample_scene(p - Vec3::new(e, 0.0, 0.0)).dist,
            self.sample_scene(p + Vec3::new(0.0, e, 0.0)).dist - self.sample_scene(p - Vec3::new(0.0, e, 0.0)).dist,
            self.sample_scene(p + Vec3::new(0.0, 0.0, e)).dist - self.sample_scene(p - Vec3::new(0.0, 0.0, e)).dist,
        )
        .normalize()
    }
}

impl SceneFrame for RayMarchingFrame {
    fn get_pixel_color(&self, coord: Vec2, _time: f32) -> Color {
        let ray = self.camera_frame.ray(coord);
        let mut marching_dist = 0.0;

        let light = Vec3::new(50.0, 10.0, -50.0);

        for _i in 0..self.max_steps {
            let marching_point = ray.at(marching_dist);
            let scene_sample = self.sample_scene(marching_point);

            if scene_sample.dist < self.hit_epsilon {
                let light_dir = (light - marching_point).normalize();
                let surface_normal = self.estimate_normal_tetrahedral(marching_point);

                let intensity = surface_normal.dot(light_dir); //.max(0.0);
                let c = if intensity < 0.0 {
                    Color::rgb(0.7, 1.0, 0.7)
                } else {
                    scene_sample.color
                };

                return c.scale_rgb(intensity.abs());
            }

            marching_dist += scene_sample.dist;

            if marching_dist > self.max_distance {
                break;
            }
        }

        Color::rgb(0.0, 0.0, 0.0)
    }
}

impl Scene for RayMarching {
    fn update(&mut self, delta_time: f32, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.camera, delta_time, input);
    }

    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        let time = time * 2.0;
        Box::new(RayMarchingFrame {
            camera_frame: self.camera.prepare_frame(),
            sphere_1: Sphere {
                center: Vec3::new(0.5 + time.sin(), 0.0, 3.0 + time.cos()),
                radius: 1.0,
                color: Color::RED,
            },

            sphere_2: Sphere {
                center: Vec3::new(-0.5 - time.sin(), 0.0, 4.0 - time.cos()),
                radius: 1.0,
                color: Color::BLUE,
            },

            max_distance: self.max_distance,
            hit_epsilon: self.hit_epsilon,
            max_steps: self.max_steps,
        })
    }
}
