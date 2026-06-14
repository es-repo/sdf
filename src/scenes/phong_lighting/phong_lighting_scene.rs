use super::phong_lighting_scene_controls::PhongLightingSceneControls;
use super::scene_object::SceneObject;
use crate::geometry::{Vec2, Vec3};
use crate::input::InputState;
use crate::rendering::{
    AmbientLight, Camera, CameraController, CameraFrame, PhongMaterial, PointLight, RayMarchResult, RayMarchSettings,
    SdfSample, estimate_normal_tetrahedral, phong_lighting, ray_march,
};
use crate::scene::{Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct PhongLightingScene {
    object: SceneObject,
    controls: PhongLightingSceneControls,
    camera: Camera,
    camera_controller: CameraController,
    ray_march_settings: RayMarchSettings,
}

impl Default for PhongLightingScene {
    fn default() -> Self {
        let fov_y = 60f32.to_radians();
        let controls = PhongLightingSceneControls::default();
        Self {
            object: SceneObject::new(Vec3::new(0.0, 0.0, 5.0), 1.0, controls.object_color),
            controls,
            camera: Camera::new(fov_y, 1.0),
            ray_march_settings: RayMarchSettings {
                max_steps: 256,
                hit_epsilon: 0.001,
                max_distance: 100.0,
                min_step: 0.005,
                near_clip: 0.05,
            },
            camera_controller: CameraController::flight(10.0),
        }
    }
}

struct PhongLightingSceneFrame {
    camera_frame: CameraFrame,
    object: SceneObject,
    light: PointLight,
    ambient_light: AmbientLight,
    material: PhongMaterial,
    ray_march_settings: RayMarchSettings,
}

impl PhongLightingSceneFrame {
    fn sample_scene(&self, point: Vec3) -> SdfSample {
        let dist = self.object.dist(&point);
        SdfSample::new(dist, self.object.color)
    }
}

impl SceneFrame for PhongLightingSceneFrame {
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
                ..self.material
            },
            self.ambient_light,
        )
    }
}

impl Scene for PhongLightingScene {
    fn update(&mut self, real_delta_time: f32, _scene_delta_time: f32, _scene_time: f32, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.camera, real_delta_time, input);
    }

    fn prepare_frame(&self, scene_time: f32) -> Box<dyn SceneFrame> {
        let mut object = self.object;
        object.color = self.controls.object_color;
        object.rotate_y(scene_time * 2.0);

        Box::new(PhongLightingSceneFrame {
            camera_frame: self.camera.prepare_frame(),
            object,

            light: PointLight {
                position: Vec3::new(50.0, 10.0, -50.0),
                color: self.controls.light_color,
                intensity: self.controls.light_intensity,
            },

            ambient_light: AmbientLight {
                color: self.controls.ambient_color,
                intensity: self.controls.ambient_intensity,
            },

            material: PhongMaterial {
                diffuse_color: self.controls.object_color,
                specular_color: self.controls.specular_color,
                specular_intensity: self.controls.specular_intensity,
                shininess: self.controls.specular_shininess,
            },

            ray_march_settings: self.ray_march_settings,
        })
    }

    fn has_controls_ui(&self) -> bool {
        true
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui) {
        self.controls.ui(ui);
    }
}
