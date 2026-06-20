use super::domain_repetition_scene_controls::DomainRepetitionSceneControls;
use crate::color_ext::ColorExt;
use crate::geometry::{Sdf3d, Sphere, Vec2, Vec3};
use crate::input::InputState;
use crate::rendering::{
    AmbientLight, Camera, CameraController, CameraFrame, ExponentialFog, PhongMaterial, PointLight, RayMarchResult,
    RayMarchSettings, SdfSample, estimate_normal_tetrahedral, phong_lighting, ray_march,
};
use crate::scene::{FrameTime, Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct DomainRepetitionScene {
    camera: Camera,
    camera_controller: CameraController,
    ray_march_settings: RayMarchSettings,
    controls: DomainRepetitionSceneControls,
    sphere: Sphere,
    ambient_light: AmbientLight,
}

impl Default for DomainRepetitionScene {
    fn default() -> Self {
        let fov_y = 60f32.to_radians();
        let mut camera = Camera::new(fov_y, 1.0);
        camera.position.z = -1.0;

        Self {
            camera,
            ray_march_settings: RayMarchSettings {
                max_steps: 256,
                hit_epsilon: 0.001,
                max_distance: 200.0,
                min_step: 0.005,
                near_clip: 0.5,
            },
            camera_controller: CameraController::flight(10.0),
            controls: DomainRepetitionSceneControls::default(),

            sphere: Sphere {
                center: Vec3::new(0.0, 0.0, 0.0),
                radius: 0.5,
                color: Color::RED,
            },

            ambient_light: AmbientLight {
                color: Color::rgb(0.7, 0.7, 0.99),
                intensity: 0.5,
            },
        }
    }
}

struct DomainRepetitionSceneFrame {
    camera_frame: CameraFrame,
    light: PointLight,
    ambient_light: AmbientLight,
    material: PhongMaterial,
    fog: ExponentialFog,
    ray_march_settings: RayMarchSettings,
    sphere: Sphere,
    controls: DomainRepetitionSceneControls,
}

impl DomainRepetitionSceneFrame {
    fn sample_scene(&self, point: Vec3, scene_time: f32) -> SdfSample {
        let (local_point, cell_index) = point.to_lattice_cell(self.controls.spacing);

        let scene_time_scaled = scene_time * 20.0;
        let vibration = Vec3::new(
            (scene_time_scaled + cell_index.x + cell_index.y).sin(),
            (scene_time_scaled + cell_index.y + cell_index.z).sin(),
            (scene_time_scaled + cell_index.z + cell_index.x).sin(),
        ) * 0.04;

        let local_point = local_point - vibration;

        let color = Color::rgb(
            0.5 + 0.5 * (cell_index.x + 1.0).sin(),
            0.5 + 0.5 * cell_index.y.sin(),
            0.5 + 0.5 * cell_index.z.sin(),
        );

        let dist = self.sphere.dist(&local_point);

        SdfSample::new(dist, color)
    }
}

impl SceneFrame for DomainRepetitionSceneFrame {
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

        let lit_color = phong_lighting(
            ray.origin,
            hit.point,
            surface_normal,
            self.light,
            PhongMaterial {
                diffuse_color: hit.sample.color,
                ..self.material
            },
            self.ambient_light,
        );

        self.fog.apply(lit_color, hit.distance)
    }
}

impl Scene for DomainRepetitionScene {
    fn update(&mut self, time: FrameTime, input: &InputState) {
        let camera_speed = 2.0;
        let camera_direction = Vec3::new(1.0, 0.5, 1.0).normalize();
        self.camera.position = self.camera.position + camera_direction * camera_speed * time.scene_time_delta;

        self.camera_controller
            .update_camera(&mut self.camera, time.real_time_delta, input);
    }

    fn prepare_frame(&self, _scene_time: f32) -> Box<dyn SceneFrame> {
        let mut ray_march_settings = self.ray_march_settings;
        ray_march_settings.max_distance = self.controls.max_distance;

        Box::new(DomainRepetitionSceneFrame {
            camera_frame: self.camera.prepare_frame(),

            light: PointLight {
                position: Vec3::new(50.0, 10.0, -50.0),
                color: Color::WHITE,
                intensity: 1.0,
            },

            ambient_light: self.ambient_light,

            material: PhongMaterial {
                diffuse_color: self.sphere.color,
                specular_color: Color::WHITE,
                specular_intensity: 1.0,
                shininess: 50.0,
            },

            fog: ExponentialFog::new(self.ambient_light.color, self.controls.fog_density),

            ray_march_settings,

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
