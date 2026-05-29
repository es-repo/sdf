use crate::color_ext::ColorExt;
use crate::geometry::{Sdf3d, Sphere, Vec2, Vec3};
use crate::input::InputState;
use crate::procedural::smooth_union::smooth_union_color;
use crate::rendering::{Camera, CameraController, CameraFrame};
use crate::scene::{Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct RayMarching {
    state: RayMarchingState,
    camera_controller: CameraController,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct RayMarchingState {
    camera: Camera,
    max_distance: f32,
    hit_epsilon: f32,
    max_steps: usize,
}

crate::stateful_scene!(RayMarching, RayMarchingState);

impl Default for RayMarching {
    fn default() -> Self {
        let fov_y = 60f32.to_radians();
        Self {
            state: RayMarchingState {
                camera: Camera::new(fov_y, 1.0),
                max_distance: 100.0,
                hit_epsilon: 0.0001,
                max_steps: 100,
            },
            camera_controller: CameraController::flight(2.0),
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
        let r = 0.2;
        let dx = r * point.x.sin();
        let dy = r * point.y.cos();

        let point = Vec3::new(point.x - dx, point.y, point.z);

        let sphere_1_dist = self.sphere_1.dist(&point);
        let sphere_2_dist = self.sphere_2.dist(&point);

        let (dist, color) = smooth_union_color(
            sphere_1_dist,
            self.sphere_1.color,
            sphere_2_dist,
            self.sphere_2.color.lerp(Color::GREEN, 0.9),
            0.5,
        );

        SdfSample::new(dist, color)
    }

    pub fn estimate_normal_tetrahedral(&self, p: Vec3, e: f32) -> Vec3 {
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

    pub fn estimate_normal_central_differences(&self, p: Vec3, e: f32) -> Vec3 {
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
                let surface_normal = self.estimate_normal_tetrahedral(marching_point, self.hit_epsilon * 2.0);

                let diffuse = surface_normal.dot(light_dir).abs();
                let intensity = 0.15 + 0.85 * diffuse;

                return scene_sample.color.scale_rgb(intensity.abs());
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
    crate::scene_state!();

    fn update(&mut self, delta_time: f32, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.state.camera, delta_time, input);
    }

    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        let time = time * 2.0;
        Box::new(RayMarchingFrame {
            camera_frame: self.state.camera.prepare_frame(),
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

            max_distance: self.state.max_distance,
            hit_epsilon: self.state.hit_epsilon,
            max_steps: self.state.max_steps,
        })
    }
}
