mod scene_1;
mod scene_2;
mod simplex_noise;
mod simplex_noise_3d;
mod smooth_union;

use crate::Vec2;
use pixels::wgpu::Color;
pub use scene_1::Scene1;
pub use scene_2::Scene2;
pub use simplex_noise::SimplexNoise;
pub use simplex_noise_3d::SimplexNoise3d;
pub use smooth_union::SmoothUnion;

pub trait SceneFrame: Send + Sync {
    fn get_pixel_color(&self, coord: Vec2<f32>, time: f32) -> Color;
}

pub trait Scene: Send + Sync {
    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame>;
}
