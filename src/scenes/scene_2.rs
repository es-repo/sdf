use crate::Vec2;
use crate::scenes::Scene;
use crate::scenes::circle::Circle;
use pixels::wgpu::Color;

pub struct Scene2;

impl Scene for Scene2 {
    fn get_pixel_color(&self, coord: Vec2<f64>, time: f64) -> Color {
        let circle_1 = Circle {
            radius: 0.3,
            center: Vec2::new(0.5 * (time * 0.5).sin(), 0.0),
            color: Color {
                r: 1.0,
                g: 0.3,
                b: 0.4,
                a: 1.0,
            },
        };

        let circle_2 = Circle {
            radius: 0.3,
            center: Vec2::new(-0.5 * (time * 0.5).cos(), 0.0),
            color: Color {
                r: 0.3,
                g: 1.0,
                b: 0.4,
                a: 1.0,
            },
        };

        let circle_1_dist = circle_1.dist(&coord);
        let circle_2_dist = circle_2.dist(&coord);

        let dist_min = circle_1_dist.min(circle_2_dist);
        let dist_max = circle_1_dist.max(circle_2_dist);

        let dist_diff = (circle_1_dist - circle_2_dist).abs();

        let (d, h) = smooth_union(circle_1_dist, circle_2_dist, 0.12);

        if d < 0.0 {
            return Color::WHITE;
            /*return Color {
                r: circle_2.color.r * (1.0 - h) + circle_1.color.r * h,
                g: circle_2.color.g * (1.0 - h) + circle_1.color.g * h,
                b: circle_2.color.b * (1.0 - h) + circle_1.color.b * h,
                a: 1.0,
            };*/
        }

        if dist_min < 0.0 {
            return Color::WHITE;
        }

        if circle_1_dist < 0.0 {
            return circle_1.color;
        }

        if circle_2_dist < 0.0 {
            return circle_2.color;
        }

        Color::BLACK
    }
}

pub fn smooth_union(d1: f64, d2: f64, k: f64) -> (f64, f64) {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    let d = d2 * (1.0 - h) + d1 * h - k * h * (1.0 - h);
    (d, h)
}
