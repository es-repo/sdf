use super::sdf_displacement_scene_controls::SdfDisplacementSceneControls;
use crate::color_ext::ColorExt;
use crate::geometry::{Sdf3d, Sphere, Vec2, Vec3};
use crate::input::InputState;
use crate::procedural::{Fbm, NoiseSimplex};
use crate::rendering::{
    AmbientLight, Camera, CameraController, CameraFrame, PhongMaterial, PointLight, RayMarchResult, RayMarchSettings,
    SdfSample, estimate_normal_tetrahedral, phong_lighting, ray_march,
};
use crate::scene::{FrameTime, Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct SdfDisplacementScene {
    camera: Camera,
    camera_controller: CameraController,
    ray_march_settings: RayMarchSettings,
    sphere: Sphere,
    controls: SdfDisplacementSceneControls,
}

impl Default for SdfDisplacementScene {
    fn default() -> Self {
        let fov_y = 60f32.to_radians();
        let sphere_center = Vec3::new(0.0, 0.0, 5.0);

        Self {
            camera: Camera::new(fov_y, 1.0),
            camera_controller: CameraController::arcball(sphere_center, 5.0),
            ray_march_settings: RayMarchSettings {
                max_steps: 256,
                hit_epsilon: 0.001,
                max_distance: 100.0,
                min_step: 0.002,
                near_clip: 0.05,
            },
            sphere: Sphere {
                center: Vec3::new(0.0, 0.0, 5.0),
                radius: 2.0,
                color: Color::rgb(0.95, 0.35, 0.12),
            },
            controls: SdfDisplacementSceneControls::default(),
        }
    }
}

struct SdfDisplacementSceneFrame {
    camera_frame: CameraFrame,
    ray_march_settings: RayMarchSettings,
    sphere: Sphere,
    rotation_y: f32,
    controls: SdfDisplacementSceneControls,
    light: PointLight,
    ambient_light: AmbientLight,
    material: PhongMaterial,
}

impl SdfDisplacementSceneFrame {
    fn displacement(&self, world_point: Vec3) -> f32 {
        let local_point = (world_point - self.sphere.center).rotate(Vec3::y_axis(), -self.rotation_y);

        (local_point * self.controls.noise_scale).fbm(
            self.controls.octaves,
            self.controls.amplitude,
            self.controls.gain,
            self.controls.lacunarity,
            |p| p.noise_simplex(),
        ) * self.controls.displacement_strength
    }

    fn sample_scene(&self, point: Vec3) -> SdfSample {
        let dist = self.sphere.dist(&point) + self.displacement(point);
        SdfSample::new(dist, self.sphere.color)
    }
}

impl SceneFrame for SdfDisplacementSceneFrame {
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

impl Scene for SdfDisplacementScene {
    fn update(&mut self, time: FrameTime, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.camera, time.real_time_delta, input);
    }

    fn prepare_frame(&self, scene_time: f32) -> Box<dyn SceneFrame> {
        Box::new(SdfDisplacementSceneFrame {
            camera_frame: self.camera.prepare_frame(),
            ray_march_settings: self.ray_march_settings,
            sphere: self.sphere,
            rotation_y: scene_time * 0.15,
            controls: self.controls,
            light: PointLight {
                position: Vec3::new(20.0, 10.0, -20.0),
                color: Color::WHITE,
                intensity: 1.0,
            },
            ambient_light: AmbientLight {
                color: Color::rgb(0.65, 0.68, 0.95),
                intensity: 0.25,
            },
            material: PhongMaterial {
                diffuse_color: self.sphere.color,
                specular_color: Color::WHITE,
                specular_intensity: 0.8,
                shininess: 40.0,
            },
        })
    }

    fn has_controls_ui(&self) -> bool {
        true
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui) {
        self.controls.ui(ui);
    }
}
