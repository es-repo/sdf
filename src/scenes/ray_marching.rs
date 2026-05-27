use crate::geometry::{Sdf3d, Sphere, Vec2, Vec3};
use crate::input::InputState;
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

impl SceneFrame for RayMarchingFrame {
    fn get_pixel_color(&self, coord: Vec2, _time: f32) -> Color {
        let ray = self.camera_frame.ray(coord);
        let mut marching_dist = 0.0;

        for _i in 0..self.max_steps {
            let marching_point = ray.at(marching_dist);
            let dist_1 = self.sphere_1.dist(&marching_point);

            if dist_1 < self.hit_epsilon {
                return self.sphere_1.color;
            }

            let dist_2 = self.sphere_2.dist(&marching_point);

            if dist_2 < self.hit_epsilon {
                return self.sphere_2.color;
            }

            let dist = dist_1.min(dist_2);

            marching_dist += dist;

            if marching_dist > self.max_distance {
                break;
            }
        }

        Color::BLACK
    }
}

impl Scene for RayMarching {
    fn update(&mut self, delta_time: f32, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.camera, delta_time, input);
    }

    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        Box::new(RayMarchingFrame {
            camera_frame: self.camera.prepare_frame(),
            sphere_1: Sphere {
                center: Vec3::new(0.5 + time.sin(), 0.0, 3.0 + time.cos()),
                radius: 1.0,
                color: Color::RED,
            },

            sphere_2: Sphere {
                center: Vec3::new(-0.5 - time.sin(), 1.5, 4.0 - time.cos()),
                radius: 2.0,
                color: Color::BLUE,
            },

            max_distance: self.max_distance,
            hit_epsilon: self.hit_epsilon,
            max_steps: self.max_steps,
        })
    }
}
