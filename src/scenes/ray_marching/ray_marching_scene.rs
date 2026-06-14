use super::ray_march_settings_controls::ray_march_settings_ui;
use crate::color_ext::ColorExt;
use crate::geometry::{Sdf3d, Sphere, Vec2, Vec3};
use crate::input::InputState;
use crate::procedural::smooth_combine::smooth_union_color;
use crate::rendering::{
    AmbientLight, Camera, CameraController, CameraFrame, PhongMaterial, PointLight, RayMarchResult, RayMarchSettings,
    SdfSample, estimate_normal_tetrahedral, phong_lighting, ray_march,
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
    //sphere_3: Sphere,
    light: PointLight,
    ambient_light: AmbientLight,
    ray_march_settings: RayMarchSettings,
}

impl RayMarchingSceneFrame {
    fn sample_scene(&self, point: Vec3) -> SdfSample {
        let r = 2.5;
        let dx = (r * point.x).sin();
        let dy = (r * point.y).cos();

        // let point = Vec3::new(point.x + dy, point.y + dx, point.z);

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

        //let dist = sphere_1_dist.min(sphere_2_dist);
        //let dist = sphere_2_dist;

        //SdfSample::new(dist, self.sphere_1.color)
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

    fn update(&mut self, real_delta_time: f32, _scene_delta_time: f32, _scene_time: f32, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.state.camera, real_delta_time, input);
    }

    fn has_controls_ui(&self) -> bool {
        true
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui) {
        ray_march_settings_ui(ui, &mut self.state.ray_march_settings, default_ray_march_settings());
    }

    fn prepare_frame(&self, scene_time: f32) -> Box<dyn SceneFrame> {
        let animation_time = scene_time * 2.0;
        Box::new(RayMarchingSceneFrame {
            camera_frame: self.state.camera.prepare_frame(),
            sphere_1: Sphere {
                center: Vec3::new(0.5 + animation_time.sin(), 0.0, 3.0 + animation_time.cos()),
                radius: 1.0,
                color: Color::GREEN,
            },

            sphere_2: Sphere {
                center: Vec3::new(-0.5 - animation_time.sin(), 0.0, 4.0 - animation_time.cos()),
                radius: 2.0,
                color: Color::BLUE,
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
