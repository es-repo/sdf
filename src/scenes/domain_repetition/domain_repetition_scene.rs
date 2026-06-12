use super::domain_repetition_scene_controls::DomainRepetitionSceneControls;
use crate::color_ext::ColorExt;
use crate::geometry::{Sdf3d, Sphere, Vec2, Vec3};
use crate::input::InputState;
use crate::rendering::{
    AmbientLight, Camera, CameraController, CameraFrame, PhongMaterial, PointLight, RayMarchResult, RayMarchSettings,
    SdfSample, estimate_normal_tetrahedral, phong_lighting, ray_march,
};
use crate::scene::{Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct DomainRepetitionScene {
    camera: Camera,
    camera_controller: CameraController,
    ray_march_settings: RayMarchSettings,
    sphere: Sphere,
    controls: DomainRepetitionSceneControls,
}

impl Default for DomainRepetitionScene {
    fn default() -> Self {
        let fov_y = 60f32.to_radians();
        Self {
            camera: Camera::new(fov_y, 1.0),
            ray_march_settings: RayMarchSettings {
                max_steps: 256,
                hit_epsilon: 0.001,
                max_distance: 200.0,
                min_step: 0.005,
            },
            camera_controller: CameraController::flight(10.0),
            sphere: Sphere {
                center: Vec3::new(0.0, 0.0, 0.0),
                radius: 0.5,
                color: Color::RED,
            },
            controls: DomainRepetitionSceneControls::default(),
        }
    }
}

struct DomainRepetitionSceneFrame {
    camera_frame: CameraFrame,
    light: PointLight,
    ambient_light: AmbientLight,
    material: PhongMaterial,
    ray_march_settings: RayMarchSettings,
    sphere: Sphere,
    controls: DomainRepetitionSceneControls,
}

impl DomainRepetitionSceneFrame {
    fn sample_scene(&self, point: Vec3) -> SdfSample {
        let point = point.repeat(self.controls.spacing);

        let dist = self.sphere.dist(&point);

        SdfSample::new(dist, self.sphere.color)
    }
}

impl SceneFrame for DomainRepetitionSceneFrame {
    fn get_pixel_color(&self, coord: Vec2, _time: f32) -> Color {
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

impl Scene for DomainRepetitionScene {
    fn update(&mut self, delta_time: f32, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.camera, delta_time, input);
    }

    fn prepare_frame(&self, _time: f32) -> Box<dyn SceneFrame> {
        Box::new(DomainRepetitionSceneFrame {
            camera_frame: self.camera.prepare_frame(),

            light: PointLight {
                position: Vec3::new(50.0, 10.0, -50.0),
                color: Color::WHITE,
                intensity: 1.0,
            },

            ambient_light: AmbientLight {
                color: Color::rgb(0.3, 0.3, 0.5),
                intensity: 0.5,
            },

            material: PhongMaterial {
                diffuse_color: self.sphere.color,
                specular_color: Color::WHITE,
                specular_intensity: 1.0,
                shininess: 50.0,
            },

            ray_march_settings: self.ray_march_settings,

            sphere: self.sphere,

            controls: self.controls,
        })
    }

    fn has_controls_ui(&self) -> bool {
        true
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui) {
        self.controls.ui(ui);
    }
}
