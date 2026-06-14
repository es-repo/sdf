use crate::audio::AudioAnalysis;
use crate::geometry::Vec2;
use pixels::wgpu::Color;

pub trait SceneFrame: Send + Sync {
    fn get_pixel_color(&self, coord: Vec2, scene_time: f32) -> Color;

    fn get_pixel_color_with_audio(&self, coord: Vec2, scene_time: f32, _audio: &AudioAnalysis) -> Color {
        self.get_pixel_color(coord, scene_time)
    }
}
