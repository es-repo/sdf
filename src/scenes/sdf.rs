use crate::color_ext::ColorExt;
use crate::geometry::{Circle, Rectangle, Triangle};
use crate::geometry::{Sdf as _, Vec2};
use crate::math::min_pair;
use crate::scenes::{Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct Sdf;

struct SdfFrame {
    circle: Circle,
    rectangle: Rectangle,
    triangle: Triangle,
}

impl SceneFrame for SdfFrame {
    fn get_pixel_color(&self, coord: Vec2<f32>, time: f32) -> Color {
        let r = (0.5 + 0.5 * time.sin()) * 0.1;

        let c_dist = self.circle.dist_round(&coord, r);
        let r_dist = self.rectangle.dist_round(&coord, r);
        let t_dist = self.triangle.dist_round(&coord, r);

        let (dist, color) = min_pair((c_dist, self.circle.color), (r_dist, self.rectangle.color));
        let (dist, color) = min_pair((dist, color), (t_dist, self.triangle.color));

        if dist < 0.0 {
            let c = Color::rgb(1.0, 1.0, 1.0);
            let t = dist.abs() / self.circle.radius;
            color.lerp(c, t)
        } else if dist < 0.02 {
            Color::rgb(1.0, 1.0, 1.0)
        } else {
            let c = Color::rgb(0.1, 0.1, (dist * 150.0).sin());

            let t = dist / 1.0;
            Color::BLACK.lerp(c, t)
        }
    }
}

impl Scene for Sdf {
    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        let time_scaled = time * 0.5;

        Box::new(SdfFrame {
            circle: Circle {
                radius: 0.2,
                center: Vec2::new(0.8 * (time_scaled + 0.2).cos(), 0.8 * time.sin()),
                color: Color::rgb(0.7, 1.0, 0.0),
            },

            rectangle: Rectangle {
                center: Vec2::new(0.8 * time.sin(), 0.8 * time_scaled.cos()),
                vertex: Vec2::new(0.3, 0.2),
                rotation: time_scaled.cos() * 0.5,
                color: Color::rgb(1.0, 0.7, 0.0),
            },

            triangle: Triangle {
                p0: Vec2::new(-0.3 + time_scaled.cos(), -0.15 + 0.3 * time_scaled.sin()).rotate(time.sin()),
                p1: Vec2::new(0.3 + time_scaled.cos(), -0.15 + 0.3 * time_scaled.sin()).rotate(time.sin()),
                p2: Vec2::new(0.0 + time_scaled.cos(), 0.4 + 0.3 * time_scaled.sin()).rotate(time.sin()),
                color: Color::rgb(1.0, 0.0, 0.7),
            },
        })
    }
}
