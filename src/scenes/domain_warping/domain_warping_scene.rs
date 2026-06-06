use super::controls::DomainWarpingSceneControls;
use crate::color_ext::ColorExt;
use crate::geometry::{Circle, Sdf, Vec2};
use crate::procedural::{Fbm, NoiseSimplex};
use crate::scene::{Scene, SceneFrame};
use pixels::wgpu::Color;

#[derive(Default)]
pub struct DomainWarpingScene {
    controls: DomainWarpingSceneControls,
}

struct DomainWarpingSceneFrame {
    circle: Circle,
    controls: DomainWarpingSceneControls,
    time_scaled: f32,
}

impl Scene for DomainWarpingScene {
    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        let circle = Circle {
            center: Vec2::new(0.0, 0.0),
            radius: 0.5,
            color: Color::rgb(0.3, 1.0, 0.4),
        };

        Box::new(DomainWarpingSceneFrame {
            circle,
            controls: self.controls,
            time_scaled: time * 0.25,
        })
    }

    fn has_controls_ui(&self) -> bool {
        true
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui) {
        self.controls.ui(ui);
    }
}

impl SceneFrame for DomainWarpingSceneFrame {
    fn get_pixel_color(&self, coord: Vec2, _time: f32) -> Color {
        let noise_coord = coord * self.controls.scale + self.time_scaled;
        let offset = noise_coord.fbm(
            self.controls.octaves,
            self.controls.amplitude,
            self.controls.gain,
            self.controls.lacunarity,
            |coord| coord.noise_simplex(),
        ) * self.controls.warp_strength;

        let warped_coord = coord + offset;

        let dist = self.circle.dist(&warped_coord);

        if dist < 0.0 { self.circle.color } else { Color::BLACK }
    }
}
