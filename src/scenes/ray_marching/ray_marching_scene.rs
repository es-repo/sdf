use super::ground::Ground;
use super::ray_march_scene_controls::RayMarchSceneControls;
use crate::color_ext::ColorExt;
use crate::geometry::{Cuboid, Cylinder, Pyramid, Quat, SignedDistance3d, Sphere, Vec2, Vec3};
use crate::input::InputState;
use crate::min_pair_many;
use crate::procedural::smooth_subtraction;
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
        camera.position.z = -8.0;
        camera.position.y = 3.0;
        let downward_pitch = Quat::from_axis_angle(Vec3::x_axis(), 10.0_f32.to_radians());

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
    sphere_1_inner_cylinder: Cylinder,
    sphere_2: Sphere,
    sphere_2_inner_cube_1: Cuboid,
    sphere_2_inner_cube_2: Cuboid,
    sphere_3: Sphere,
    cuboid: Cuboid,
    pyramid: Pyramid,
    cylinder: Cylinder,
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
        let r = 0.1;

        let ground_sample = self.ground.sample(point, scene_time, self.animate_ground);

        let sphere_1_dist = self.sphere_1.dist(point);
        let sphere_1_inner_cylinder = self.sphere_1_inner_cylinder.dist(point);
        let (sphere_1_dist, _) = smooth_subtraction(sphere_1_dist, sphere_1_inner_cylinder, r * 2.0);

        let sphere_2_dist = self.sphere_2.dist(point);
        let sphere_2_inner_cube_1 = self.sphere_2_inner_cube_1.dist(point);
        let sphere_2_inner_cube_2 = self.sphere_2_inner_cube_2.dist(point);
        let (sphere_2_dist, _) = smooth_subtraction(sphere_2_dist, sphere_2_inner_cube_1, r * 2.0);
        let (sphere_2_dist, _) = smooth_subtraction(sphere_2_dist, sphere_2_inner_cube_2, r * 2.0);

        let sphere_3_dist = self.sphere_3.dist(point);

        let cuboid_dist = self.cuboid.dist_round(point, r);
        let pyramid_dist = self.pyramid.dist_round(point, r);
        let cylinder_dist = self.cylinder.dist_round(point, r);

        let (dist, color) = min_pair_many!(
            (sphere_1_dist, self.sphere_1.color),
            (sphere_2_dist, self.sphere_2.color),
            (sphere_3_dist, self.sphere_3.color),
            (cuboid_dist, self.cuboid.color),
            (pyramid_dist, self.pyramid.color),
            (cylinder_dist, self.cylinder.color),
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
        let animation_time = scene_time * 0.5;
        let sphere_radius = 1.0;
        let objects_y = 2.0;
        let object_remoteness = 4.0;
        let object_angle = PI * 2.0 / 6.0;
        let ambient_light = AmbientLight {
            color: Color::rgb(0.7, 0.7, 1.0),
            intensity: 0.75,
        };

        Box::new(RayMarchingSceneFrame {
            camera_frame: self.state.camera.prepare_frame(),

            sphere_1: Sphere {
                center: Vec3::new(
                    object_remoteness * (object_angle + animation_time).sin(),
                    objects_y,
                    object_remoteness * (object_angle + animation_time).cos(),
                ),
                radius: sphere_radius,
                color: Color::rgb(1.0, 0.25, 0.25),
            },

            sphere_1_inner_cylinder: Cylinder {
                center: Vec3::new(
                    object_remoteness * (object_angle + animation_time).sin(),
                    objects_y,
                    object_remoteness * (object_angle + animation_time).cos(),
                ),
                radius: 0.4,
                half_height: sphere_radius,
                rotation: Quat::from_axis_angle(Vec3::x_axis(), scene_time * 1.3).normalize(),
                color: Color::rgb(1.0, 1.0, 0.0),
            },

            sphere_2: Sphere {
                center: Vec3::new(
                    object_remoteness * (object_angle * 4.0 + animation_time).sin(),
                    objects_y,
                    object_remoteness * (object_angle * 4.0 + animation_time).cos(),
                ),
                radius: sphere_radius,
                color: Color::rgb(0.5, 1.0, 0.0),
            },

            sphere_2_inner_cube_1: Cuboid {
                center: Vec3::new(
                    object_remoteness * (object_angle * 4.0 + animation_time).sin(),
                    objects_y,
                    object_remoteness * (object_angle * 4.0 + animation_time).cos(),
                ),
                half_size: Vec3::new(sphere_radius * 2.0, sphere_radius * 0.3, sphere_radius * 0.3),
                rotation: Quat::from_axis_angle(Vec3::y_axis(), scene_time * 2.3).normalize(),
                color: Color::rgb(0.5, 0.5, 0.5),
            },

            sphere_2_inner_cube_2: Cuboid {
                center: Vec3::new(
                    object_remoteness * (object_angle * 4.0 + animation_time).sin(),
                    objects_y,
                    object_remoteness * (object_angle * 4.0 + animation_time).cos(),
                ),
                half_size: Vec3::new(sphere_radius * 0.3, sphere_radius * 0.3, sphere_radius * 2.0),
                rotation: Quat::from_axis_angle(Vec3::y_axis(), scene_time * 2.3).normalize(),
                color: Color::rgb(0.5, 0.5, 0.5),
            },

            sphere_3: Sphere {
                center: Vec3::new(
                    object_remoteness * (object_angle * 3.0 + animation_time).sin(),
                    objects_y,
                    object_remoteness * (object_angle * 3.0 + animation_time).cos(),
                ),
                radius: sphere_radius,
                color: Color::rgb(0.0, 0.5, 1.0),
            },

            cuboid: Cuboid {
                center: Vec3::new(
                    object_remoteness * (object_angle * 2.0 + animation_time).sin(),
                    objects_y,
                    object_remoteness * (object_angle * 2.0 + animation_time).cos(),
                ),
                half_size: Vec3::new(0.6, 0.6, 0.8),
                rotation: (Quat::from_axis_angle(Vec3::y_axis(), scene_time * 0.8)
                    * Quat::from_axis_angle(Vec3::x_axis(), scene_time * 0.6)
                    * Quat::from_axis_angle(Vec3::z_axis(), scene_time * 0.4))
                .normalize(),
                color: Color::rgb(1.0, 0.5, 0.2),
            },

            pyramid: Pyramid {
                center: Vec3::new(
                    object_remoteness * (object_angle * 5.0 + animation_time).sin(),
                    objects_y - 0.75,
                    object_remoteness * (object_angle * 5.0 + animation_time).cos(),
                ),
                height: 1.5,
                base_half_size: 0.75,
                rotation: Quat::from_axis_angle(Vec3::y_axis(), scene_time * 3.3).normalize(),
                color: Color::rgb(1.0, 0.2, 0.6),
            },

            cylinder: Cylinder {
                center: Vec3::new(
                    object_remoteness * (object_angle * 6.0 + animation_time).sin(),
                    objects_y,
                    object_remoteness * (object_angle * 6.0 + animation_time).cos(),
                ),
                radius: 0.7,
                half_height: 0.9,
                rotation: (Quat::from_axis_angle(Vec3::x_axis(), scene_time * 0.7)
                    * Quat::from_axis_angle(Vec3::z_axis(), scene_time * 0.5))
                .normalize(),
                color: Color::rgb(1.0, 1.0, 0.0),
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
