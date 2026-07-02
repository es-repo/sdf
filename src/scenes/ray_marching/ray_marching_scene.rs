use super::ground::Ground;
use super::ray_march_scene_controls::RayMarchSceneControls;
use crate::color_ext::ColorExt;
use crate::geometry::{Quat, SignedDistance3d, Sphere, Vec2, Vec3};
use crate::input::InputState;
use crate::min_pair_many;
use crate::rendering::{
    AmbientLight, Camera, CameraController, CameraFrame, ExponentialFog, PhongMaterial, PointLight, RayMarchResult,
    RayMarchSettings, SdfSample, estimate_normal_tetrahedral, phong_lighting, ray_march,
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
    #[serde(default)]
    controls: RayMarchSceneControls,
}

crate::stateful_scene!(RayMarchingScene, RayMarchingSceneState);

impl Default for RayMarchingScene {
    fn default() -> Self {
        let fov_y = 60f32.to_radians();
        let mut camera = Camera::new(fov_y, 1.0);
        camera.position.z = -10.0;
        camera.position.y = 8.0;
        let downward_pitch = Quat::from_axis_angle(Vec3::x_axis(), 25.0_f32.to_radians());

        camera.set_rotation(downward_pitch);

        Self {
            state: RayMarchingSceneState {
                camera,
                controls: RayMarchSceneControls::default(),
            },
            camera_controller: CameraController::flight(10.0),
        }
    }
}

struct RayMarchingSceneFrame {
    camera_frame: CameraFrame,
    sphere_1: Sphere,
    sphere_2: Sphere,
    sphere_3: Sphere,
    ground: Ground,
    light: PointLight,
    ambient_light: AmbientLight,
    fog: ExponentialFog,
    animate_ground: bool,
    show_convergence_failure_debug: bool,
    ray_march_settings: RayMarchSettings,
}

impl RayMarchingSceneFrame {
    fn sample_scene(&self, point: Vec3, scene_time: f32) -> SdfSample {
        let ground_sample = self.ground.sample(point, scene_time, self.animate_ground);

        let sphere_1_dist = self.sphere_1.dist(point);
        let sphere_2_dist = self.sphere_2.dist(point);
        let sphere_3_dist = self.sphere_3.dist(point);

        let (dist, color) = min_pair_many!(
            (sphere_1_dist, self.sphere_1.color),
            (sphere_2_dist, self.sphere_2.color),
            (sphere_3_dist, self.sphere_3.color),
            (ground_sample.dist, ground_sample.color)
        );

        SdfSample::new(dist, color)
    }
}

impl SceneFrame for RayMarchingSceneFrame {
    fn get_pixel_color(&self, coord: Vec2, scene_time: f32) -> Color {
        let ray = self.camera_frame.ray(coord);

        let hit = match ray_march(ray, self.ray_march_settings, |point| {
            self.sample_scene(point, scene_time)
        }) {
            RayMarchResult::Hit(hit) => hit,
            RayMarchResult::Miss(miss) => {
                if self.show_convergence_failure_debug && miss.steps >= self.ray_march_settings.max_steps {
                    return Color::rgb(1.0, 0.0, 1.0); // convergence failure debug
                }

                return self.ambient_light.color;
            }
        };

        let surface_normal = estimate_normal_tetrahedral(hit.point, self.ray_march_settings.hit_epsilon, |point| {
            self.sample_scene(point, scene_time).dist
        });

        let lit_color = phong_lighting(
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
        );

        self.fog.apply(lit_color, hit.distance)
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
        self.state.controls.ui(ui);
    }

    fn prepare_frame(&self, scene_time: f32) -> Box<dyn SceneFrame> {
        let animation_time = scene_time * 1.0;
        let sphere_radius = 1.0;
        let sphere_remoteness = 3.0;
        let sphere_angle = PI * 2.0 / 3.0;
        let ambient_light = AmbientLight {
            color: Color::rgb(0.7, 0.7, 1.0),
            intensity: 0.75,
        };

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

            ground: Ground::new(Color::rgb(0.7, 0.7, 0.7), 0.0, 10.0),

            light: PointLight {
                position: Vec3::new(50.0, 10.0, -50.0),
                color: Color::WHITE,
                intensity: 1.0,
            },

            ambient_light,

            fog: ExponentialFog::new(ambient_light.color, 0.1, 50.0),

            animate_ground: self.state.controls.animate_ground,

            show_convergence_failure_debug: self.state.controls.show_convergence_failure_debug,

            ray_march_settings: self.state.controls.ray_march_settings,
        })
    }
}
