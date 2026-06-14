use pixels::wgpu::Color;

use crate::color_ext::ColorExt;
use crate::geometry::{Circle, Vec2};
use crate::procedural::smooth_combine::smooth_union;
use crate::procedural::{Fbm, NoiseSimplex};
use crate::scene::{Scene, SceneFrame};

pub struct Scene2Scene;

struct Scene2SceneFrame {
    circle_1: Circle,
    circle_2: Circle,
    time_sin: f32,
}

impl SceneFrame for Scene2SceneFrame {
    fn get_pixel_color(&self, coord: Vec2, _scene_time: f32) -> Color {
        let d1 = self.circle_1.dist_squared_radius_squared(&coord);
        let d2 = self.circle_2.dist_squared_radius_squared(&coord);

        let mut circle_1_dist = d1.sin() * d1;
        let circle_2_dist = d2.sin() * d1;

        circle_1_dist = circle_1_dist.min(circle_2_dist);

        let (d, _h) = smooth_union(circle_1_dist, circle_2_dist, 0.3);
        circle_1_dist = d;

        if circle_1_dist < 0.0 {
            self.circle_1.color
        } else {
            let f = coord.fbm_rotated(4, 0.5, 0.5, |coord| coord.noise_simplex());

            let value = 0.5 + 0.5 * f;
            let mut color = Color::rgba(value, value, value, 0.0);

            if circle_1_dist < 0.1 {
                let value = (-circle_1_dist * 50.0).exp();
                color = Color::rgba(value, value, value, value);
            }

            let wave = 0.5 + (circle_1_dist * 10.0 - self.time_sin * 5.0).sin();

            color.blend(Color::rgb(0.5 + 0.6 * wave * f, 0.5 + 0.6 * wave * f, 0.5 + 0.6 * f))
        }
    }
}

impl Scene for Scene2Scene {
    fn prepare_frame(&self, scene_time: f32) -> Box<dyn SceneFrame> {
        let time_sin = scene_time.sin();
        let time_cos = scene_time.cos();
        let time2_sin = (scene_time * 2.0).sin();
        let time2_cos = (scene_time * 2.0).cos();

        Box::new(Scene2SceneFrame {
            circle_1: Circle {
                radius: 0.0,
                center: Vec2::new(0.5 * time2_sin, 0.35 * time_sin * time2_cos),
                color: Color::rgb(1.0, 0.9, 0.9),
            },
            circle_2: Circle {
                radius: 0.0,
                center: Vec2::new(-0.5 * time2_sin, 0.35 * time_cos * time2_sin),
                color: Color::rgb(1.0, 0.3, 0.4),
            },
            time_sin,
        })
    }
}
