use pixels::wgpu::Color;

use crate::scenes::circle::Circle;
use crate::scenes::{Scene, SceneFrame};
use crate::{ColorExt, Fbm, NoiseSimplex, Vec2, smooth_union};

pub struct Scene2;

struct Scene2Frame {
    circle_1: Circle,
    circle_2: Circle,
    time_sin: f32,
}

impl SceneFrame for Scene2Frame {
    fn get_pixel_color(&self, coord: Vec2<f32>, _time: f32) -> Color {
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

            let mut color = Color {
                r: (0.5 + 0.5 * f) as f64,
                g: (0.5 + 0.5 * f) as f64,
                b: (0.5 + 0.5 * f) as f64,
                a: 0.0,
            };

            if circle_1_dist < 0.1 {
                color = Color {
                    r: ((-circle_1_dist * 50.0).exp()) as f64,
                    g: ((-circle_1_dist * 50.0).exp()) as f64,
                    b: ((-circle_1_dist * 50.0).exp()) as f64,
                    a: ((-circle_1_dist * 50.0).exp()) as f64,
                };
            }

            let wave = 0.5 + (circle_1_dist * 10.0 - self.time_sin * 5.0).sin();

            color.blend(Color {
                r: (0.5 + 0.6 * wave * f) as f64,
                g: (0.5 + 0.6 * wave * f) as f64,
                b: (0.5 + 0.6 * f) as f64,
                a: 1.0,
            });

            color
        }
    }
}

impl Scene for Scene2 {
    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        let time_sin = time.sin();
        let time_cos = time.cos();
        let time2_sin = (time * 2.0).sin();
        let time2_cos = (time * 2.0).cos();

        Box::new(Scene2Frame {
            circle_1: Circle {
                radius: 0.0,
                center: Vec2::new(0.5 * time2_sin, 0.35 * time_sin * time2_cos),
                color: Color {
                    r: 1.0,
                    g: 0.9,
                    b: 0.9,
                    a: 1.0,
                },
            },
            circle_2: Circle {
                radius: 0.0,
                center: Vec2::new(-0.5 * time2_sin, 0.35 * time_cos * time2_sin),
                color: Color {
                    r: 1.0,
                    g: 0.3,
                    b: 0.4,
                    a: 1.0,
                },
            },
            time_sin,
        })
    }
}
