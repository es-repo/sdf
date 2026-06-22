#[derive(Clone, Copy)]
pub struct SdfDisplacementSceneControls {
    pub displacement_strength: f32,
    pub noise_scale: f32,
    pub octaves: u32,
    pub amplitude: f32,
    pub gain: f32,
    pub lacunarity: f32,
}

impl Default for SdfDisplacementSceneControls {
    fn default() -> Self {
        Self {
            displacement_strength: 0.04,
            noise_scale: 3.0,
            octaves: 4,
            amplitude: 0.5,
            gain: 0.5,
            lacunarity: 2.0,
        }
    }
}

impl SdfDisplacementSceneControls {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.displacement_strength, 0.0..=0.3).text("Strength"));
        ui.add(egui::Slider::new(&mut self.noise_scale, 0.2..=12.0).text("Scale"));
        ui.add(egui::Slider::new(&mut self.octaves, 1..=8).text("Octaves"));
        ui.add(egui::Slider::new(&mut self.amplitude, 0.0..=1.0).text("Amplitude"));
        ui.add(egui::Slider::new(&mut self.gain, 0.0..=1.0).text("Gain"));
        ui.add(egui::Slider::new(&mut self.lacunarity, 1.0..=4.0).text("Lacunarity"));

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}
