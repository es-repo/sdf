use crate::geometry::Vec2;
use crate::scene::{Scene, SceneFrame};
use pixels::wgpu::Color;

pub struct DomainRepetitionScene;

struct DomainRepetitionSceneFrame;

impl SceneFrame for DomainRepetitionSceneFrame {
    fn get_pixel_color(&self, _coord: Vec2, _time: f32) -> Color {
        Color::BLACK
    }
}

impl Scene for DomainRepetitionScene {
    fn prepare_frame(&self, _time: f32) -> Box<dyn SceneFrame> {
        Box::new(DomainRepetitionSceneFrame)
    }
}
