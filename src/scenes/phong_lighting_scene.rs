use crate::color_ext::ColorExt;
use crate::geometry::{Sdf3d, Sphere, Vec2, Vec3};
use crate::input::InputState;
use crate::procedural::smooth_union::smooth_union_many;
use crate::rendering::{
    AmbientLight, Camera, CameraController, CameraFrame, PhongMaterial, PointLight, RayMarchResult, RayMarchSettings,
    SdfSample, estimate_normal_tetrahedral, phong_lighting, ray_march,
};
use crate::scene::{Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct PhongLightingScene {
    camera: Camera,
    camera_controller: CameraController,
    ray_march_settings: RayMarchSettings,
}

impl Default for PhongLightingScene {
    fn default() -> Self {
        let fov_y = 60f32.to_radians();
        Self {
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

struct Object {
    position: Vec3,
    spheres: [Sphere; 5],
    pub color: Color,
}

impl Object {
    pub fn new(position: Vec3, core_radius: f32, color: Color) -> Self {
        let side_radius = core_radius * 0.8;
        let side_shift = core_radius * 1.2;

        let spheres = [
            Sphere {
                center: Vec3::new(0.0, 0.0, 0.0),
                radius: 1.0,
                color,
            },
            Sphere {
                center: Vec3::new(-side_shift, side_shift, 0.0),
                radius: side_radius,
                color,
            },
            Sphere {
                center: Vec3::new(-side_shift, -side_shift, 0.0),
                radius: side_radius,
                color,
            },
            Sphere {
                center: Vec3::new(side_shift, -side_shift, 0.0),
                radius: side_radius,
                color,
            },
            Sphere {
                center: Vec3::new(side_shift, side_shift, 0.0),
                radius: side_radius,
                color,
            },
        ];

        Self {
            position,
            spheres,
            color,
        }
    }

    pub fn dist(&self, point: &Vec3) -> f32 {
        let distances = self.spheres.map(|s| s.dist(&(*point - self.position)));
        smooth_union_many(distances, 0.5).unwrap()
    }
}

struct PhongLightingSceneFrame {
    camera_frame: CameraFrame,
    object: Object,
    light: PointLight,
    ambient_light: AmbientLight,
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
                specular_color: Color::WHITE,
                shininess: 10.0,
            },
            self.ambient_light,
        )
    }
}

impl Scene for PhongLightingScene {
    fn update(&mut self, delta_time: f32, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.camera, delta_time, input);
    }

    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        let time = time * 2.0;

        Box::new(PhongLightingSceneFrame {
            camera_frame: self.camera.prepare_frame(),
            object: Object::new(Vec3::new(0.0, 0.0, 4.0), 1.0, Color::rgb(0.4, 0.3, 1.0)),

            light: PointLight {
                position: Vec3::new(50.0, 10.0, -50.0),
                color: Color::WHITE,
                intensity: 1.0,
            },

            ambient_light: AmbientLight {
                color: Color::rgb(0.7, 0.7, 1.0),
                intensity: 0.75,
            },

            ray_march_settings: self.ray_march_settings,
        })
    }
}
