use super::object::Object;
use crate::color_ext::ColorExt;
use crate::geometry::{Vec2, Vec3};
use crate::input::InputState;
use crate::rendering::{
    AmbientLight, Camera, CameraController, CameraFrame, PhongMaterial, PointLight, RayMarchResult, RayMarchSettings,
    SdfSample, estimate_normal_tetrahedral, phong_lighting, ray_march,
};
use crate::scene::{Scene, SceneFrame};
use pixels::wgpu::Color;

#[derive(Clone, Copy)]
pub struct PhongLightingSceneParams {
    light_color: Color,
    light_intensity: f32,
    ambient_color: Color,
    ambient_intensity: f32,
    object_color: Color,
    specular_color: Color,
    specular_shininess: f32,
}

impl Default for PhongLightingSceneParams {
    fn default() -> Self {
        Self {
            light_color: Color::WHITE,
            light_intensity: 1.0,
            ambient_color: Color::rgb(0.7, 0.7, 1.0),
            ambient_intensity: 0.75,
            object_color: Color::rgb(1.0, 0.5, 0.0),
            specular_color: Color::WHITE,
            specular_shininess: 50.0,
        }
    }
}

pub struct PhongLightingScene {
    object: Object,
    params: PhongLightingSceneParams,
    camera: Camera,
    camera_controller: CameraController,
    ray_march_settings: RayMarchSettings,
}

impl Default for PhongLightingScene {
    fn default() -> Self {
        let fov_y = 60f32.to_radians();
        let params = PhongLightingSceneParams::default();
        Self {
            object: Object::new(Vec3::new(0.0, 0.0, 5.0), 1.0, params.object_color),
            params,
            camera: Camera::new(fov_y, 1.0),
            ray_march_settings: RayMarchSettings {
                max_steps: 100,
                hit_epsilon: 0.0001,
                max_distance: 100.0,
                min_step: 0.005,
            },
            camera_controller: CameraController::flight(10.0),
        }
    }
}

struct PhongLightingSceneFrame {
    camera_frame: CameraFrame,
    object: Object,
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

impl Scene for PhongLightingScene {
    fn update(&mut self, delta_time: f32, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.camera, delta_time, input);

        self.object.rotate_y(delta_time * 2.0);
    }

    fn prepare_frame(&self, _time: f32) -> Box<dyn SceneFrame> {
        let mut object = self.object;
        object.color = self.params.object_color;

        Box::new(PhongLightingSceneFrame {
            camera_frame: self.camera.prepare_frame(),
            object,

            light: PointLight {
                position: Vec3::new(50.0, 10.0, -50.0),
                color: self.params.light_color,
                intensity: self.params.light_intensity,
            },

            ambient_light: AmbientLight {
                color: self.params.ambient_color,
                intensity: self.params.ambient_intensity,
            },

            material: PhongMaterial {
                diffuse_color: self.params.object_color,
                specular_color: self.params.specular_color,
                shininess: self.params.specular_shininess,
            },

            ray_march_settings: self.ray_march_settings,
        })
    }

    fn has_parameters_ui(&self) -> bool {
        true
    }

    fn parameters_ui(&mut self, ui: &mut egui::Ui) {
        color_control(ui, "Object color", &mut self.params.object_color);
        ui.separator();

        color_control(ui, "Light color", &mut self.params.light_color);
        ui.add(egui::Slider::new(&mut self.params.light_intensity, 0.0..=5.0).text("Intensity"));

        ui.separator();

        color_control(ui, "Ambient color", &mut self.params.ambient_color);
        ui.add(egui::Slider::new(&mut self.params.ambient_intensity, 0.0..=2.0).text("Intensity"));

        ui.separator();

        color_control(ui, "Specular color", &mut self.params.specular_color);
        ui.add(egui::Slider::new(&mut self.params.specular_shininess, 1.0..=128.0).text("Shininess"));

        if ui.button("Reset").clicked() {
            self.params = PhongLightingSceneParams::default();
        }
    }
}

fn color_control(ui: &mut egui::Ui, label: &str, color: &mut Color) {
    ui.horizontal(|ui| {
        let mut rgb = [color.r as f32, color.g as f32, color.b as f32];

        if ui.color_edit_button_rgb(&mut rgb).changed() {
            *color = Color::rgb(rgb[0], rgb[1], rgb[2]);
        }

        ui.label(label);
    });
}
