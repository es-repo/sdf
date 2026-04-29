use crate::scenes::{Scene, SceneFrame};
use crate::{Circle, Fbm, NoiseSimplex, Vec2};
use pixels::wgpu::Color;

pub struct DomainWarping;

struct DomainWarpingFrame {
    circle: Circle,
    time_scaled: f32,
}

impl Scene for DomainWarping {
    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        let circle = Circle {
            center: Vec2::new(0.0, 0.0),
            radius: 0.5,
            color: Color::GREEN,
        };

        Box::new(DomainWarpingFrame {
            circle,
            time_scaled: time * 0.25,
        })
    }
}

impl SceneFrame for DomainWarpingFrame {
    fn get_pixel_color(&self, coord: Vec2<f32>, _time: f32) -> Color {
        let scale = 3.0;
        let warp_strength = 0.05;
        let octaves = 4;

        let noise_coord = coord * scale + self.time_scaled;
        let offset = noise_coord.fbm_rotated(octaves, 0.5, 0.5, |coord| coord.noise_simplex()) * warp_strength;
        let warped_coord = coord + offset;

        let dist = self.circle.dist(&warped_coord);

        if dist < 0.0 { self.circle.color } else { Color::BLACK }
    }
}
