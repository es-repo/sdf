use super::scene_7_controls::Scene7Controls;
use super::scene_object::SceneObject;
use crate::color_ext::ColorExt;
use crate::geometry::{Sdf3d, Vec2, Vec3};
use crate::input::InputState;
use crate::rendering::{
    AmbientLight, Camera, CameraController, CameraFrame, PhongMaterial, PointLight, RayMarchResult, RayMarchSettings,
    SdfSample, estimate_normal_tetrahedral, phong_lighting, ray_march,
};
use crate::scene::{FrameTime, Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct Scene7 {
    state: Scene7State,
    camera_controller: CameraController,
    controls: Scene7Controls,
    scene_object: SceneObject,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Scene7State {
    camera: Camera,
    ray_march_settings: RayMarchSettings,
}

impl Default for Scene7 {
    fn default() -> Self {
        let fov_y = 60f32.to_radians();
        let controls = Scene7Controls::default();
        Self {
            state: Scene7State {
                camera: Camera::new(fov_y, 1.0),
                ray_march_settings: default_ray_march_settings(),
            },
            camera_controller: CameraController::arcball(Vec3::new(0.0, 0.0, 0.0), 5.0),
            controls,
            scene_object: SceneObject::new(
                Vec3::new(0.0, 0.0, 0.0),
                2.0,
                controls.object_color,
                controls.spheres_per_ring,
            ),
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

struct Scene7Frame {
    camera_frame: CameraFrame,
    scene_object: SceneObject,
    light: PointLight,
    ambient_light: AmbientLight,
    ray_march_settings: RayMarchSettings,
    union_color: Color,
    subtraction_color: Color,
}

impl Scene7Frame {
    fn sample_scene(&self, point: Vec3) -> SdfSample {
        let dist = self.scene_object.dist(point);
        let core_sphere_dist = self.scene_object.core_sphere.dist(point);
        let k = 0.8;
        let dist_diff = core_sphere_dist * k - dist * k;
        let color = if dist_diff > 0.0 {
            self.union_color
        } else {
            self.subtraction_color
        };
        let dist_diff = dist_diff.abs();

        let h = dist_diff.clamp(0.0, 1.0);
        let color = self.scene_object.color.lerp(color, h);

        SdfSample::new(dist, color)
    }
}

impl SceneFrame for Scene7Frame {
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
    }
}

impl Scene for Scene7 {
    fn update(&mut self, time: FrameTime, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.state.camera, time.real_time_delta, input);
    }

    fn has_controls_ui(&self) -> bool {
        true
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui) {
        self.controls.ui(ui);
        self.scene_object.set_spheres_per_ring(self.controls.spheres_per_ring);
    }

    fn prepare_frame(&self, scene_time: f32) -> Box<dyn SceneFrame> {
        let mut scene_object = self.scene_object.clone();
        scene_object.color = self.controls.object_color;
        scene_object.update(scene_time);

        Box::new(Scene7Frame {
            camera_frame: self.state.camera.prepare_frame(),

            scene_object,

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
            union_color: self.controls.out_color,
            subtraction_color: self.controls.in_color,
        })
    }
}
