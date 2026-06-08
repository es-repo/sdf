use crate::color_ext::ColorExt;
use crate::geometry::{Circle, Sdf, Vec2};
use crate::procedural::smooth_combine::smooth_union;
use crate::scene::{Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct SmoothUnionScene;

struct SmoothUnionSceneFrame {
    circle_1: Circle,
    circle_2: Circle,
}

impl SceneFrame for SmoothUnionSceneFrame {
    fn get_pixel_color(&self, coord: Vec2, _time: f32) -> Color {
        let circle_1_dist = self.circle_1.dist(&coord);
        let circle_2_dist = self.circle_2.dist(&coord);

        let (d, h) = smooth_union(circle_1_dist, circle_2_dist, 0.12);

        if d < 0.0 {
            return self.circle_2.color.lerp(self.circle_1.color, h);
        }

        Color::BLACK
    }
}

impl Scene for SmoothUnionScene {
    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        let time_scaled = time * 0.5;

        Box::new(SmoothUnionSceneFrame {
            circle_1: Circle {
                radius: 0.3,
                center: Vec2::new(0.5 * time_scaled.sin(), 0.0),
                color: Color::rgb(1.0, 0.3, 0.4),
            },
            circle_2: Circle {
                radius: 0.3,
                center: Vec2::new(-0.5 * time_scaled.cos(), 0.0),
                color: Color::rgb(0.3, 1.0, 0.4),
            },
        })
    }
}
