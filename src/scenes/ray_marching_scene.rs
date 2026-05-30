use crate::color_ext::ColorExt;
use crate::geometry::{Sdf3d, Sphere, Vec2, Vec3};
use crate::input::InputState;
use crate::procedural::smooth_union::smooth_union_color;
use crate::rendering::{
    Camera, CameraController, CameraFrame, RayMarchResult, RayMarchSettings, SdfSample, estimate_normal_tetrahedral,
    ray_march,
};
use crate::scene::{Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct RayMarchingScene {
    state: RayMarchingSceneState,
    camera_controller: CameraController,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct RayMarchingSceneState {
    camera: Camera,
    ray_march_settings: RayMarchSettings,
}

crate::stateful_scene!(RayMarchingScene, RayMarchingSceneState);

impl Default for RayMarchingScene {
    fn default() -> Self {
        let fov_y = 60f32.to_radians();
        Self {
            state: RayMarchingSceneState {
                camera: Camera::new(fov_y, 1.0),
                ray_march_settings: RayMarchSettings {
                    max_steps: 100,
                    hit_epsilon: 0.0001,
                    max_distance: 100.0,
                },
            },
            camera_controller: CameraController::flight(2.0),
        }
    }
}

struct RayMarchingSceneFrame {
    camera_frame: CameraFrame,
    sphere_1: Sphere,
    sphere_2: Sphere,
    //sphere_3: Sphere,
    ray_march_settings: RayMarchSettings,
}

impl RayMarchingSceneFrame {
    fn sample_scene(&self, point: Vec3) -> SdfSample {
        let r = 3.5;
        let dx = (r * point.x).sin();
        let dy = (r * point.y).cos();

        let point = Vec3::new(point.x + dy, point.y + dx, point.z);

        let sphere_1_dist = self.sphere_1.dist(&point);
        let sphere_2_dist = self.sphere_2.dist(&point);

        let (dist, color) = smooth_union_color(
            sphere_1_dist,
            self.sphere_1.color,
            sphere_2_dist,
            self.sphere_2.color,
            0.5,
        );

        SdfSample::new(dist, color)
    }
}

impl SceneFrame for RayMarchingSceneFrame {
    fn get_pixel_color(&self, coord: Vec2, _time: f32) -> Color {
        let ray = self.camera_frame.ray(coord);

        let light = Vec3::new(50.0, 10.0, -50.0);

        let hit = match ray_march(ray, self.ray_march_settings, |point| self.sample_scene(point)) {
            RayMarchResult::Hit(hit) => hit,
            RayMarchResult::Miss(_) => return Color::rgb(0.0, 0.0, 0.0),
        };

        let light_dir = (light - hit.point).normalize();
        let surface_normal =
            estimate_normal_tetrahedral(hit.point, self.ray_march_settings.hit_epsilon * 2.0, |point| {
                self.sample_scene(point).dist
            });

        let diffuse = surface_normal.dot(light_dir).abs();
        let intensity = 0.15 + 0.85 * diffuse;

        hit.sample.color.scale_rgb(intensity.abs())
    }
}

impl Scene for RayMarchingScene {
    crate::scene_state!();

    fn update(&mut self, delta_time: f32, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.state.camera, delta_time, input);
    }

    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        let time = time * 2.0;
        Box::new(RayMarchingSceneFrame {
            camera_frame: self.state.camera.prepare_frame(),
            sphere_1: Sphere {
                center: Vec3::new(0.5 + time.sin(), 0.0, 3.0 + time.cos()),
                radius: 1.0,
                color: Color::GREEN,
            },

            sphere_2: Sphere {
                center: Vec3::new(-0.5 - time.sin(), 0.0, 4.0 - time.cos()),
                radius: 1.0,
                color: Color::BLUE,
            },

            ray_march_settings: self.state.ray_march_settings,
        })
    }
}
