use pixels::wgpu::Color;

use crate::scenes::{Scene, SceneFrame};
use crate::{Fbm, NoiseSimplex, Vec2};

pub struct SimplexNoise;

struct SimplexNoiseFrame {}

impl Scene for SimplexNoise {
    fn prepare_frame(&self, _time: f32) -> Box<dyn SceneFrame> {
        Box::new(SimplexNoiseFrame {})
    }
}

impl SceneFrame for SimplexNoiseFrame {
    fn get_pixel_color(&self, coord: Vec2<f32>, time: f32) -> Color {
        let mut f;
        let scale = 3.0;
        let time_scaled = time * 0.2;

        if coord.x < 0.0 && coord.y > 0.0 {
            let coord = coord * scale + time_scaled;
            f = coord.noise_simplex();
        } else {
            let octaves = if coord.x > 0.0 && coord.y > 0.0 {
                2
            } else if coord.x < 0.0 && coord.y < 0.0 {
                3
            } else {
                4
            };

            let coord = coord * scale + time_scaled;
            f = coord.fbm_rotated(octaves, 0.5, 0.5);
        }

        f = 0.5 + 0.5 * f;

        Color {
            r: f as f64,
            g: f as f64,
            b: f as f64,
            a: 1.0,
        }
    }
}
