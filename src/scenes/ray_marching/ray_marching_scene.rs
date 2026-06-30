use super::ray_march_settings_controls::ray_march_settings_ui;
use crate::color_ext::ColorExt;
use crate::geometry::{Plane, SignedDistance3d, Sphere, Vec2, Vec3};
use crate::input::InputState;
use crate::min_pair_many;
use crate::rendering::{
    AmbientLight, Camera, CameraController, CameraFrame, PhongMaterial, PointLight, RayMarchResult, RayMarchSettings,
    SdfSample, estimate_normal_tetrahedral, phong_lighting, ray_march,
};
use crate::scene::{FrameTime, Scene, SceneFrame};
use pixels::wgpu::Color;
use std::f32::consts::PI;

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
                ray_march_settings: default_ray_march_settings(),
            },
            camera_controller: CameraController::flight(10.0),
        }
    }
}

fn default_ray_march_settings() -> RayMarchSettings {
    RayMarchSettings {
        max_steps: 100,
        hit_epsilon: 0.0001,
        max_distance: 100.0,
        min_step: 0.005,
        near_clip: 0.05,
    }
}

struct RayMarchingSceneFrame {
    camera_frame: CameraFrame,
    sphere_1: Sphere,
    sphere_2: Sphere,
    sphere_3: Sphere,
    floor: Plane,
    light: PointLight,
    ambient_light: AmbientLight,
    ray_march_settings: RayMarchSettings,
}

impl RayMarchingSceneFrame {
    fn sample_scene(&self, point: Vec3) -> SdfSample {
        let sphere_1_dist = self.sphere_1.dist(point);
        let sphere_2_dist = self.sphere_2.dist(point);
        let sphere_3_dist = self.sphere_3.dist(point);
        let floor_dist = self.floor.dist(point);

        let (dist, color) = min_pair_many!(
            (sphere_1_dist, self.sphere_1.color),
            (sphere_2_dist, self.sphere_2.color),
            (sphere_3_dist, self.sphere_3.color),
            (floor_dist, self.floor.color)
        );

        SdfSample::new(dist, color)
    }
}

impl SceneFrame for RayMarchingSceneFrame {
    fn get_pixel_color(&self, coord: Vec2, _scene_time: f32) -> Color {
        let ray = self.camera_frame.ray(coord);

        let hit = match ray_march(ray, self.ray_march_settings, |point| self.sample_scene(point)) {
            RayMarchResult::Hit(hit) => hit,
            RayMarchResult::Miss(miss) => {
                if miss.steps >= self.ray_march_settings.max_steps {
                    return Color::rgb(1.0, 0.0, 1.0); // convergence failure debug
                }

                return self.ambient_light.color;
            }
        };

        let surface_normal = estimate_normal_tetrahedral(hit.point, self.ray_march_settings.hit_epsilon, |point| {
            self.sample_scene(point).dist
        });

        phong_lighting(
            ray.origin,
            hit.point,
            surface_normal,
            self.light,
            PhongMaterial {
                diffuse_color: hit.sample.color,
                specular_color: Color::WHITE,
                specular_intensity: 1.0,
                shininess: 10.0,
            },
            self.ambient_light,
        )
    }
}

impl Scene for RayMarchingScene {
    crate::scene_state!();

    fn update(&mut self, time: FrameTime, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.state.camera, time.real_time_delta, input);
    }

    fn has_controls_ui(&self) -> bool {
        true
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui) {
        ray_march_settings_ui(ui, &mut self.state.ray_march_settings, default_ray_march_settings());
    }

    fn prepare_frame(&self, scene_time: f32) -> Box<dyn SceneFrame> {
        let animation_time = scene_time * 1.0;
        let sphere_radius = 1.0;
        let sphere_remoteness = 3.0;
        let sphere_angle = PI * 2.0 / 3.0;

        Box::new(RayMarchingSceneFrame {
            camera_frame: self.state.camera.prepare_frame(),

            sphere_1: Sphere {
                center: Vec3::new(
                    sphere_remoteness * (sphere_angle + animation_time).sin(),
                    1.5,
                    sphere_remoteness * (sphere_angle + animation_time).cos(),
                ),
                radius: sphere_radius,
                color: Color::rgb(1.0, 0.0, 0.0),
            },

            sphere_2: Sphere {
                center: Vec3::new(
                    sphere_remoteness * (sphere_angle * 2.0 + animation_time).sin(),
                    sphere_radius,
                    sphere_remoteness * (sphere_angle * 2.0 + animation_time).cos(),
                ),
                radius: sphere_radius,
                color: Color::rgb(0.0, 1.0, 0.0),
            },

            sphere_3: Sphere {
                center: Vec3::new(
                    sphere_remoteness * (sphere_angle * 3.0 + animation_time).sin(),
                    sphere_radius,
                    sphere_remoteness * (sphere_angle * 3.0 + animation_time).cos(),
                ),
                radius: sphere_radius,
                color: Color::rgb(0.0, 0.0, 1.0),
            },

            floor: Plane {
                normal: Vec3::new(0.0, 1.0, 0.0),
                offset: 0.0,
                color: Color::rgb(0.7, 0.7, 0.7),
            },

            light: PointLight {
                position: Vec3::new(50.0, 10.0, -50.0),
                color: Color::WHITE,
                intensity: 1.0,
            },

            ambient_light: AmbientLight {
                color: Color::rgb(0.7, 0.7, 1.0),
                intensity: 0.75,
            },

            ray_march_settings: self.state.ray_march_settings,
        })
    }
}
