mod circle;
mod scene_1;
mod scene_2;
mod simplex_noise;
mod smooth_union;

use crate::Vec2;
use pixels::wgpu::Color;
pub use scene_1::Scene1;
pub use scene_2::Scene2;
pub use simplex_noise::SimplexNoise;
pub use smooth_union::SmoothUnion;

pub trait SceneFrame: Send + Sync {
    fn get_pixel_color(&self, coord: Vec2<f64>, time: f64) -> Color;
}

pub trait Scene: Send + Sync {
    fn prepare_frame(&self, time: f64) -> Box<dyn SceneFrame>;
}
