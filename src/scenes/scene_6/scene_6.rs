use super::scene_6_controls::Scene6Controls;
use super::scene_object::SceneObject;
use crate::color_ext::ColorExt;
use crate::geometry::{Vec2, Vec3};
use crate::input::InputState;
use crate::rendering::{
    AmbientLight, Camera, CameraController, CameraFrame, PhongMaterial, PointLight, RayMarchResult, RayMarchSettings,
    SdfSample, estimate_normal_tetrahedral, phong_lighting, ray_march,
};
use crate::scene::{FrameTime, Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct Scene6 {
    state: Scene6State,
    camera_controller: CameraController,
    controls: Scene6Controls,
    scene_object: SceneObject,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Scene6State {
    camera: Camera,
    ray_march_settings: RayMarchSettings,
}

impl Default for Scene6 {
    fn default() -> Self {
        let fov_y = 60f32.to_radians();
        let controls = Scene6Controls::default();
        Self {
            state: Scene6State {
                camera: Camera::new(fov_y, 1.0),
                ray_march_settings: default_ray_march_settings(),
            },
            camera_controller: CameraController::arcball(Vec3::new(0.0, 0.0, 0.0), 5.0),
            controls,
            scene_object: SceneObject::new(Vec3::new(0.0, 0.0, -0.5), 1.0, controls.object_color),
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

struct Scene6Frame {
    camera_frame: CameraFrame,
    scene_object: SceneObject,
    light: PointLight,
    ambient_light: AmbientLight,
    ray_march_settings: RayMarchSettings,
    step_blend_color: Color,
    step_blend_threshold: f32,
    displacement_strength: f32,
    noise_scale: f32,
}

impl Scene6Frame {
    fn sample_scene(&self, point: Vec3) -> SdfSample {
        let dist = self
            .scene_object
            .dist(&point, self.displacement_strength, self.noise_scale);
        SdfSample::new(dist, self.scene_object.color)
    }
}

impl SceneFrame for Scene6Frame {
    fn get_pixel_color(&self, coord: Vec2, _scene_time: f32) -> Color {
        let ray = self.camera_frame.ray(coord);

        let hit = match ray_march(ray, self.ray_march_settings, |point| self.sample_scene(point)) {
            RayMarchResult::Hit(hit) => hit,
            RayMarchResult::Miss(_) => {
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
        .lerp(
            self.step_blend_color,
            (hit.steps as f32 / (self.ray_march_settings.max_steps as f32 * self.step_blend_threshold)).clamp(0.0, 1.0),
        )
    }
}

impl Scene for Scene6 {
    fn update(&mut self, time: FrameTime, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.state.camera, time.real_time_delta, input);
    }

    fn has_controls_ui(&self) -> bool {
        true
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui) {
        self.controls.ui(ui);
    }

    fn prepare_frame(&self, scene_time: f32) -> Box<dyn SceneFrame> {
        let mut scene_object = self.scene_object;
        scene_object.color = self.controls.object_color;
        scene_object.rotate_y(scene_time * 0.5);

        Box::new(Scene6Frame {
            camera_frame: self.state.camera.prepare_frame(),

            scene_object,

            light: PointLight {
                position: Vec3::new(50.0, 10.0, -50.0),
                color: Color::WHITE,
                intensity: 1.0,
            },

            ambient_light: AmbientLight {
                color: self.controls.ambient_color,
                intensity: 0.75,
            },

            ray_march_settings: self.state.ray_march_settings,
            step_blend_color: self.controls.step_blend_color,
            step_blend_threshold: self.controls.step_blend_threshold,
            displacement_strength: self.controls.displacement_strength,
            noise_scale: self.controls.noise_scale,
        })
    }
}
