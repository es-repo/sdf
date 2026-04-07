use pixels::wgpu::Color;

use crate::Vec2;
use crate::scenes::{Scene, SceneFrame};

pub struct SimplexNoise;

struct SimplexNoiseFrame {
    time: f64,
}

impl SceneFrame for SimplexNoiseFrame {
    fn get_pixel_color(&self, coord: Vec2<f64>, _time: f64) -> Color {
        let mut f;

        if coord.x < 0.0 && coord.y > 0.0 {
            let coord = coord * 5.0 + self.time;
            f = coord.noise_simplex();
        } else {
            let octaves = if coord.x > 0.0 && coord.y > 0.0 {
                1
            } else if coord.x < 0.0 && coord.y < 0.0 {
                2
            } else {
                3
            };

            let coord = coord * 5.0 + self.time;
            f = coord.fbm_with_transform(
                octaves,
                0.5,
                0.5,
                |coord| coord.noise_simplex(),
                |coord| Vec2::new(1.6 * coord.x + 1.2 * coord.y, -1.2 * coord.x + 1.6 * coord.y),
            );
        }

        f = 0.5 + 0.5 * f;

        Color {
            r: f,
            g: f,
            b: f,
            a: 1.0,
        }
    }
}

impl Scene for SimplexNoise {
    fn prepare_frame(&self, time: f64) -> Box<dyn SceneFrame> {
        Box::new(SimplexNoiseFrame { time })
    }
}
