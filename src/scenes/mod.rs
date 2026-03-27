mod circle;
mod scene_1;
mod scene_2;

use crate::Vec2;
use pixels::wgpu::Color;
pub use scene_1::Scene1;
pub use scene_2::Scene2;

pub trait Scene {
    fn get_pixel_color(&self, coord: Vec2<f64>, time: f64) -> Color;
}
