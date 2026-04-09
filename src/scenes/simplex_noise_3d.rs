use crate::scenes::{Scene, SceneFrame};
use crate::{Vec2, Vec3};
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

        let coord = Vec3::from_2d(coord + time, time);
        f = coord.noise_simplex();

        f = 0.5 + 0.5 * f;

        Color {
            r: f as f64,
            g: f as f64,
            b: f as f64,
            a: 1.0,
        }
    }
}
