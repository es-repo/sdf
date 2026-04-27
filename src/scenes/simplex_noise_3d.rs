use crate::scenes::{Scene, SceneFrame};
use crate::{Fbm, NoiseSimplex, Vec2, Vec3};
use pixels::wgpu::Color;

pub struct SimplexNoise3d;

struct SimplexNoise3dFrame {}

impl Scene for SimplexNoise3d {
    fn prepare_frame(&self, _time: f32) -> Box<dyn SceneFrame> {
        Box::new(SimplexNoise3dFrame {})
    }
}

impl SceneFrame for SimplexNoise3dFrame {
    fn get_pixel_color(&self, coord: Vec2<f32>, time: f32) -> Color {
        let mut f;
        let scale = 3.0;
        let time_scaled = time * 0.2;

        if coord.x < 0.0 && coord.y > 0.0 {
            let coord3d = Vec3::from_2d(coord * scale + time_scaled, time_scaled);
            f = coord3d.noise_simplex();
        } else {
            let octaves = if coord.x > 0.0 && coord.y > 0.0 {
                2
            } else if coord.x < 0.0 && coord.y < 0.0 {
                3
            } else {
                4
            };

            let coord3d = Vec3::from_2d(coord * scale + time_scaled, time_scaled);
            f = coord3d.fbm_rotated(octaves, 0.5, 0.5);
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
