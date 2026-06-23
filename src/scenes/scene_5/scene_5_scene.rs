use super::scene_5_scene_controls::Scene5SceneControls;
use crate::color_ext::ColorExt;
use crate::geometry::{Sdf3d, Sphere, Vec2, Vec3};
use crate::procedural::{Fbm, NoiseSimplex};
use crate::rendering::{
    AmbientLight, Camera, CameraFrame, PhongMaterial, PointLight, RayMarchResult, RayMarchSettings, SdfSample,
    estimate_normal_tetrahedral, phong_lighting, ray_march,
};
use crate::scene::{Scene, SceneFrame};
use pixels::wgpu::Color;

const DISPLACEMENT_STRENGTH: f32 = 0.110;
const NOISE_SCALE: f32 = 2.5;
const OCTAVES: u32 = 3;
const AMPLITUDE: f32 = 0.78;
const GAIN: f32 = 0.09;
const LACUNARITY: f32 = 4.0;

pub struct Scene5Scene {
    camera: Camera,
    ray_march_settings: RayMarchSettings,
    sphere: Sphere,
    controls: Scene5SceneControls,
}

impl Default for Scene5Scene {
    fn default() -> Self {
        let sphere_center = Vec3::new(0.0, 0.0, 5.0);
        let mut camera = Camera::new(60f32.to_radians(), 1.0);
        camera.position = Vec3::new(0.0, 0.0, 2.15);

        let controls = Scene5SceneControls::default();

        Self {
            camera,
            ray_march_settings: RayMarchSettings {
                max_steps: 256,
                hit_epsilon: 0.001,
                max_distance: 100.0,
                min_step: 0.002,
                near_clip: 0.05,
            },
            sphere: Sphere {
                center: sphere_center,
                radius: 2.2,
                color: controls.sphere_color,
            },
            controls,
        }
    }
}

struct Scene5SceneFrame {
    camera_frame: CameraFrame,
    ray_march_settings: RayMarchSettings,
    sphere: Sphere,
    rotation_y: f32,
    light: PointLight,
    ambient_light: AmbientLight,
    material: PhongMaterial,
}

impl Scene5SceneFrame {
    fn displacement(&self, world_point: Vec3, scene_time: f32) -> f32 {
        let mut local_point = (world_point - self.sphere.center).rotate(Vec3::y_axis(), -self.rotation_y);
        local_point.z += scene_time * 0.1;

        (local_point * NOISE_SCALE).fbm(OCTAVES, AMPLITUDE, GAIN, LACUNARITY, |p| p.noise_simplex())
            * DISPLACEMENT_STRENGTH
    }

    fn sample_scene(&self, point: Vec3, scene_time: f32) -> SdfSample {
        let dist = self.sphere.dist(&point) + self.displacement(point, scene_time);
        SdfSample::new(dist, self.sphere.color)
    }
}

impl SceneFrame for Scene5SceneFrame {
    fn get_pixel_color(&self, coord: Vec2, scene_time: f32) -> Color {
        let ray = self.camera_frame.ray(coord);

        let hit = match ray_march(ray, self.ray_march_settings, |point| {
            self.sample_scene(point, scene_time)
        }) {
            RayMarchResult::Hit(hit) => hit,
            RayMarchResult::Miss(_) => {
                return self.ambient_light.color;
            }
        };

        let surface_normal = estimate_normal_tetrahedral(hit.point, self.ray_march_settings.hit_epsilon, |point| {
            self.sample_scene(point, scene_time).dist
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
        .gamma_corrected()
    }
}

impl Scene for Scene5Scene {
    fn prepare_frame(&self, scene_time: f32) -> Box<dyn SceneFrame> {
        let mut sphere = self.sphere;
        sphere.color = self.controls.sphere_color;

        Box::new(Scene5SceneFrame {
            camera_frame: self.camera.prepare_frame(),
            ray_march_settings: self.ray_march_settings,
            sphere,
            rotation_y: scene_time * 0.025,
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
                diffuse_color: sphere.color,
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
