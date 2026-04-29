use crate::scenes::{ParameterizedScene, Scene, SceneFrame};
use crate::{Fbm, NoiseSimplex, Vec2, Vec3};
use pixels::wgpu::Color;

#[derive(Clone, Copy)]
pub struct Scene3Params {
    pub scale: f32,
    pub amplitude: f32,
    pub gain: f32,
    pub octaves: u32,
    pub warp_iterations: u32,
}

impl Default for Scene3Params {
    fn default() -> Self {
        Self {
            scale: 3.0,
            amplitude: 0.5,
            gain: 0.5,
            octaves: 4,
            warp_iterations: 2,
        }
    }
}

#[derive(Default)]
pub struct Scene3 {
    params: Scene3Params,
}

struct Scene3Frame {
    params: Scene3Params,
}

impl Scene for Scene3 {
    fn prepare_frame(&self, _time: f32) -> Box<dyn SceneFrame> {
        Box::new(Scene3Frame { params: self.params })
    }
}

impl ParameterizedScene for Scene3 {
    fn parameters_ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.params.scale, 0.1..=16.0).text("Scale"));
        ui.add(egui::Slider::new(&mut self.params.amplitude, 0.0..=2.0).text("Amplitude"));
        ui.add(egui::Slider::new(&mut self.params.gain, 0.0..=1.0).text("Gain"));
        ui.add(egui::Slider::new(&mut self.params.octaves, 1..=8).text("Octaves"));
        ui.add(egui::Slider::new(&mut self.params.warp_iterations, 1..=8).text("Warp iterations"));

        if ui.button("Reset").clicked() {
            self.params = Scene3Params::default();
        }
    }
}

impl SceneFrame for Scene3Frame {
    fn get_pixel_color(&self, coord: Vec2<f32>, time: f32) -> Color {
        let time_scaled = time * 0.2;

        let mut coord3d = Vec3::from_2d(coord * self.params.scale + time_scaled, time_scaled);

        let mut f = 1.0;
        for _i in 0..self.params.warp_iterations {
            f = coord3d.fbm_rotated(self.params.octaves, self.params.amplitude, self.params.gain, |coord| {
                coord.noise_simplex()
            });
            coord3d = coord3d + f;
            coord3d = coord3d.sin();
        }

        f = 0.5 + 0.5 * f;

        Color {
            r: (f * f) as f64,
            g: f as f64,
            b: f as f64,
            a: 1.0,
        }
    }
}
