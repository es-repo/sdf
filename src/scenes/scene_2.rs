use pixels::wgpu::Color;

use crate::scenes::Scene;
use crate::scenes::circle::Circle;
use crate::{ColorExt, Vec2, smooth_union};

pub struct Scene2;

impl Scene for Scene2 {
    fn get_pixel_color(&self, coord: Vec2<f64>, time: f64) -> Color {
        let circle_1 = Circle {
            radius: 0.0, //0.1 + 0.2 * time.sin(),
            center: Vec2::new(0.5 * (time * 2.0).sin(), 0.35 * (time * 1.0).sin() * (time * 2.0).cos()),
            color: Color {
                r: 1.0,
                g: 0.9,
                b: 0.9,
                a: 1.0,
            },
        };

        let circle_2 = Circle {
            radius: 0.0, //0.1 + 0.2 * time.cos(),
            center: Vec2::new(
                -0.5 * (time * 2.0).sin(),
                0.35 * (time * 1.0).cos() * (time * 2.0).sin(),
            ),
            color: Color {
                r: 1.0,
                g: 0.3,
                b: 0.4,
                a: 1.0,
            },
        };

        let mut circle_1_dist =
            circle_1.dist_squared_radius_squared(&coord).sin() * circle_1.dist_squared_radius_squared(&coord);
        let circle_2_dist =
            circle_2.dist_squared_radius_squared(&coord).sin() * circle_1.dist_squared_radius_squared(&coord);

        circle_1_dist = circle_1_dist.min(circle_2_dist);

        let (d, _h) = smooth_union(circle_1_dist, circle_2_dist, 0.3);
        circle_1_dist = d;

        if circle_1_dist < 0.0 {
            circle_1.color
        } else {
            let f = coord.fbm_with_transform(
                4,
                0.5,
                0.5,
                |coord| coord.noise_simplex(),
                |coord| Vec2::new(1.6 * coord.x + 1.2 * coord.y, -1.2 * coord.x + 1.6 * coord.y),
            );

            let mut color = Color {
                r: 0.5 + 0.5 * f,
                g: 0.5 + 0.5 * f,
                b: 0.5 + 0.5 * f,
                a: 0.0,
            };

            if circle_1_dist < 0.1 {
                color = Color {
                    r: (-circle_1_dist * 50.0).exp(),
                    g: (-circle_1_dist * 50.0).exp(),
                    b: (-circle_1_dist * 50.0).exp(),
                    a: (-circle_1_dist * 50.0).exp(),
                };
            }

            let wave = 0.5 + (circle_1_dist * 10.0 - (time.sin()) * 5.0).sin();

            color.blend(Color {
                r: 0.5 + 0.5 * wave * f,
                g: 0.5 + 0.5 * wave * f,
                b: 0.5 + 0.5 * f,
                a: 1.0,
            });

            color
        }
    }
}
