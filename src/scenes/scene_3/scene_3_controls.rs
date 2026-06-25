#[derive(Clone, Copy)]
pub struct Scene3Controls {
    pub scale: f32,
    pub amplitude: f32,
    pub gain: f32,
    pub octaves: u32,
    pub warp_iterations: u32,
}

impl Default for Scene3Controls {
    fn default() -> Self {
        Self {
            scale: 2.0,
            amplitude: 0.5,
            gain: 0.5,
            octaves: 4,
            warp_iterations: 4,
        }
    }
}

impl Scene3Controls {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.scale, 0.1..=8.0).text("Scale"));
        ui.add(egui::Slider::new(&mut self.amplitude, 0.0..=2.0).text("Amplitude"));
        ui.add(egui::Slider::new(&mut self.gain, 0.0..=1.0).text("Gain"));
        ui.add(egui::Slider::new(&mut self.octaves, 1..=8).text("Octaves"));
        ui.add(egui::Slider::new(&mut self.warp_iterations, 1..=8).text("Warp iterations"));

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}
