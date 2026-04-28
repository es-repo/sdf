use crate::scenes::{Scene, SceneFrame};
use crate::{Circle, Vec2, smooth_union};
use pixels::wgpu::Color;

pub struct SmoothUnion;

struct SmoothUnionFrame {
    circle_1: Circle,
    circle_2: Circle,
}

impl SceneFrame for SmoothUnionFrame {
    fn get_pixel_color(&self, coord: Vec2<f32>, _time: f32) -> Color {
        let circle_1_dist = self.circle_1.dist(&coord);
        let circle_2_dist = self.circle_2.dist(&coord);

        let (d, h) = smooth_union(circle_1_dist, circle_2_dist, 0.12);

        if d < 0.0 {
            let h = h as f64;
            return Color {
                r: self.circle_2.color.r * (1.0 - h) + self.circle_1.color.r * h,
                g: self.circle_2.color.g * (1.0 - h) + self.circle_1.color.g * h,
                b: self.circle_2.color.b * (1.0 - h) + self.circle_1.color.b * h,
                a: 1.0,
            };
        }

        Color::BLACK
    }
}

impl Scene for SmoothUnion {
    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        let half_time = time * 0.5;

        Box::new(SmoothUnionFrame {
            circle_1: Circle {
                radius: 0.3,
                center: Vec2::new(0.5 * half_time.sin(), 0.0),
                color: Color {
                    r: 1.0,
                    g: 0.3,
                    b: 0.4,
                    a: 1.0,
                },
            },
            circle_2: Circle {
                radius: 0.3,
                center: Vec2::new(-0.5 * half_time.cos(), 0.0),
                color: Color {
                    r: 0.3,
                    g: 1.0,
                    b: 0.4,
                    a: 1.0,
                },
            },
        })
    }
}
