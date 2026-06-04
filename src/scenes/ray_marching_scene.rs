use crate::color_ext::ColorExt;
use crate::geometry::{Sdf3d, Sphere, Vec2, Vec3};
use crate::input::InputState;
use crate::procedural::smooth_union::smooth_union_color;
use crate::rendering::{
    Camera, CameraController, CameraFrame, RayMarchResult, RayMarchSettings, SdfSample, estimate_normal_tetrahedral,
    ray_march,
};
use crate::scene::{Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct RayMarchingScene {
    state: RayMarchingSceneState,
    camera_controller: CameraController,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct RayMarchingSceneState {
    camera: Camera,
    ray_march_settings: RayMarchSettings,
}

crate::stateful_scene!(RayMarchingScene, RayMarchingSceneState);

impl Default for RayMarchingScene {
    fn default() -> Self {
        let fov_y = 60f32.to_radians();
        Self {
            state: RayMarchingSceneState {
                camera: Camera::new(fov_y, 1.0),
                ray_march_settings: RayMarchSettings {
                    max_steps: 100,
                    hit_epsilon: 0.0001,
                    max_distance: 100.0,
                },
            },
            camera_controller: CameraController::flight(10.0),
        }
    }
}

struct RayMarchingSceneFrame {
    camera_frame: CameraFrame,
    sphere_1: Sphere,
    sphere_2: Sphere,
    //sphere_3: Sphere,
    ray_march_settings: RayMarchSettings,
}

impl RayMarchingSceneFrame {
    fn sample_scene(&self, point: Vec3) -> SdfSample {
        let r = 2.5;
        let dx = (r * point.x).sin();
        let dy = (r * point.y).cos();

        // let point = Vec3::new(point.x + dy, point.y + dx, point.z);

        let sphere_1_dist = self.sphere_1.dist(&point);
        let sphere_2_dist = self.sphere_2.dist(&point);

        let (dist, color) = smooth_union_color(
            sphere_1_dist,
            self.sphere_1.color.lerp(Color::RED, 0.5 + 0.5 * point.y.sin()),
            sphere_2_dist,
            self.sphere_2
                .color
                .lerp(Color::rgb(1.0, 1.0, 0.0), 0.5 + 0.5 * point.x.sin()),
            0.5,
        );

        //let dist = sphere_1_dist.min(sphere_2_dist);
        //let dist = sphere_2_dist;

        //SdfSample::new(dist, self.sphere_1.color)
        SdfSample::new(dist, color)
    }
}

impl SceneFrame for RayMarchingSceneFrame {
    fn get_pixel_color(&self, coord: Vec2, _time: f32) -> Color {
        let ray = self.camera_frame.ray(coord);

        let light_position = Vec3::new(50.0, 10.0, -50.0);
        let light_color = Color::WHITE;

        let back_color = Color::rgb(0.7, 0.7, 1.0);

        let hit = match ray_march(ray, self.ray_march_settings, |point| self.sample_scene(point)) {
            RayMarchResult::Hit(hit) => hit,
            RayMarchResult::Miss(miss) => {
                if miss.steps >= self.ray_march_settings.max_steps {
                    return Color::rgb(1.0, 0.0, 1.0); // convergence failure debug
                }

                return back_color;
            }
        };

        let surface_normal = estimate_normal_tetrahedral(hit.point, self.ray_march_settings.hit_epsilon, |point| {
            self.sample_scene(point).dist
        });

        phong_lighting(
            ray.origin,
            hit.point,
            surface_normal,
            light_position,
            light_color,
            hit.sample.color,
            Color::WHITE,
            10.0,
            back_color,
            0.45,
        )
    }
}

fn phong_lighting(
    camera_position: Vec3,
    surface_point: Vec3,
    surface_normal: Vec3,
    light_position: Vec3,
    light_color: Color,
    material_color: Color,
    material_specular_color: Color,
    material_specular_shininess: f32,
    ambient_light_color: Color,
    ambient_light_strength: f32,
) -> Color {
    let light_dir = (light_position - surface_point).normalize();

    let reflected_light_dir = (light_dir * -1.0).reflect(surface_normal);
    let view_dir = (camera_position - surface_point).normalize();

    let diffuse_strength = surface_normal.dot(light_dir).max(0.0);

    let ambient_color = material_color
        .multiply_rgb(ambient_light_color)
        .scale_rgb(ambient_light_strength);

    let diffuse_color = material_color.multiply_rgb(light_color).scale_rgb(diffuse_strength);

    let specular_strength = if diffuse_strength > 0.0 {
        reflected_light_dir
            .dot(view_dir)
            .max(0.0)
            .powf(material_specular_shininess)
    } else {
        0.0
    };

    let specular_color = material_specular_color
        .multiply_rgb(light_color)
        .scale_rgb(specular_strength);

    ambient_color.add_rgb(diffuse_color).add_rgb(specular_color)
}

impl Scene for RayMarchingScene {
    crate::scene_state!();

    fn update(&mut self, delta_time: f32, input: &InputState) {
        self.camera_controller
            .update_camera(&mut self.state.camera, delta_time, input);
    }

    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        let time = time * 2.0;
        Box::new(RayMarchingSceneFrame {
            camera_frame: self.state.camera.prepare_frame(),
            sphere_1: Sphere {
                center: Vec3::new(0.5 + time.sin(), 0.0, 3.0 + time.cos()),
                radius: 1.0,
                color: Color::GREEN,
            },

            sphere_2: Sphere {
                center: Vec3::new(-0.5 - time.sin(), 0.0, 4.0 - time.cos()),
                radius: 2.0,
                color: Color::BLUE,
            },

            ray_march_settings: self.state.ray_march_settings,
        })
    }
}
