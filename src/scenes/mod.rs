mod circle;
mod scene_1;
mod simplex_noise;
mod smooth_union;

use crate::Vec2;
use pixels::wgpu::Color;
pub use scene_1::Scene1;
pub use simplex_noise::SimplexNoise;
pub use smooth_union::SmoothUnion;

pub trait Scene {
    fn get_pixel_color(&self, coord: Vec2<f64>, time: f64) -> Color;
}
