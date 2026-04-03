use pixels::wgpu::Color;

use crate::scenes::Scene;
use crate::{ColorExt, Vec2, noise, warp};

pub struct SimplexNoise;

impl Scene for SimplexNoise {
    fn get_pixel_color(&self, coord: Vec2<f64>, time: f64) -> Color {
        let drift = Vec2::new(time * 0.15, -time * 0.08);
        let warped = warp(coord + drift, 3.0, 0.35);

        let base_noise = noise(warped * 2.5 + Vec2::new(time * 0.4, -time * 0.3));
        let detail_noise = noise(warped * 5.0 + Vec2::new(-time * 0.2, time * 0.25));

        let bands = 0.5 + 0.5 * (warped.len() * 8.0 - time * 1.5 + base_noise * 2.0).sin();
        let mist = (detail_noise * 0.5 + 0.5).clamp(0.0, 1.0);
        let glow = (0.35 - coord.len() + base_noise * 0.1).clamp(0.0, 1.0);

        let mut color = Color {
            r: 0.08 + 0.15 * mist,
            g: 0.12 + 0.35 * bands,
            b: 0.2 + 0.6 * mist,
            a: 1.0,
        };

        color.blend(Color {
            r: 0.7 * glow + 0.2 * bands,
            g: 0.8 * glow + 0.1 * mist,
            b: glow,
            a: 0.35 + 0.4 * glow,
        });

        color
    }
}
