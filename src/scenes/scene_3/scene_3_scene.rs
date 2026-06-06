use super::controls::Scene3SceneControls;
use crate::color_ext::ColorExt;
use crate::geometry::{Vec2, Vec3};
use crate::procedural::{Fbm, NoiseSimplex};
use crate::scene::{Scene, SceneFrame};
use pixels::wgpu::Color;

#[derive(Default)]
pub struct Scene3Scene {
    controls: Scene3SceneControls,
}

struct Scene3SceneFrame {
    controls: Scene3SceneControls,
    time_scaled: f32,
}

impl Scene for Scene3Scene {
    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        Box::new(Scene3SceneFrame {
            controls: self.controls,
            time_scaled: time * 0.2,
        })
    }

    fn has_controls_ui(&self) -> bool {
        true
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui) {
        self.controls.ui(ui);
    }
}

impl SceneFrame for Scene3SceneFrame {
    fn get_pixel_color(&self, coord: Vec2, _time: f32) -> Color {
        let mut coord3d = Vec3::from_2d(coord * self.controls.scale + self.time_scaled, self.time_scaled);

        let mut f = 1.0;
        for _i in 0..self.controls.warp_iterations {
            f = coord3d.fbm_rotated(
                self.controls.octaves,
                self.controls.amplitude,
                self.controls.gain,
                |coord| coord.noise_simplex(),
            );
            coord3d = coord3d + f;
            coord3d = coord3d.sin();
        }

        f = 0.5 + 0.5 * f;

        Color::rgb(f * f, f, f)
    }
}
