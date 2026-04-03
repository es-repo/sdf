use pixels::wgpu::Color;

use crate::Vec2;
use crate::scenes::Scene;

pub struct SimplexNoise;

impl Scene for SimplexNoise {
    fn get_pixel_color(&self, coord: Vec2<f64>, time: f64) -> Color {
        let octaves = if coord.x < 0.0 && coord.y > 0.0 {
            1
        } else if coord.x > 0.0 && coord.y > 0.0 {
            2
        } else if coord.x < 0.0 && coord.y < 0.0 {
            3
        } else {
            4
        };

        let coord = coord * 5.0 + time;
        let f = coord.fbm_with_transform(
            octaves,
            0.5,
            0.5,
            |coord| coord.noise_simplex(),
            |coord| Vec2::new(1.6 * coord.x + 1.2 * coord.y, -1.2 * coord.x + 1.6 * coord.y),
        );

        let f = 0.5 + 0.5 * f;

        Color {
            r: f,
            g: f,
            b: f,
            a: 1.0,
        }
    }
}
