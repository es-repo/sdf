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
            camera_controller: CameraController::flight(10.0),
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
        let r = 2.5;
        let dx = (r * point.x).sin();
        let dy = (r * point.y).cos();

        let point = Vec3::new(point.x + dy, point.y + dx, point.z);

        let sphere_1_dist = self.sphere_1.dist(&point);
        let sphere_2_dist = self.sphere_2.dist(&point);

        let (dist, color) = smooth_union_color(
            sphere_1_dist,
            self.sphere_1.color.lerp(Color::RED, 0.5 + 0.5 * point.y.sin()),
            sphere_2_dist,
            self.sphere_2
                .color
                .lerp(Color::rgb(1.0, 1.0, 0.0), 0.5 + 0.5 * point.x.sin()),
            0.5,
        );

        SdfSample::new(dist, color)
    }
}

impl SceneFrame for RayMarchingSceneFrame {
    fn get_pixel_color(&self, coord: Vec2, _time: f32) -> Color {
        let ray = self.camera_frame.ray(coord);

        let light_source = Vec3::new(50.0, 10.0, -50.0);

        let hit = match ray_march(ray, self.ray_march_settings, |point| self.sample_scene(point)) {
            RayMarchResult::Hit(hit) => hit,
            RayMarchResult::Miss(_) => return Color::rgb(0.0, 0.0, 0.0),
        };

        let light_dir = (light_source - hit.point).normalize();

        let surface_normal =
            estimate_normal_tetrahedral(hit.point, self.ray_march_settings.hit_epsilon * 2.0, |point| {
                self.sample_scene(point).dist
            });

        let reflected_light_dir = (light_dir * -1.0).reflect(surface_normal);
        let view_dir = (ray.origin - hit.point).normalize();

        let ambient = 0.15;
        let diffuse = surface_normal.dot(light_dir).max(0.0);
        let specular = reflected_light_dir.dot(view_dir).max(0.0).powf(10.0);

        let intensity = ambient + diffuse;
        let shaded_color = hit.sample.color.scale_rgb(intensity);
        let specular_color = Color::WHITE.scale_rgb(specular);

        shaded_color.add_rgb(specular_color)
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
