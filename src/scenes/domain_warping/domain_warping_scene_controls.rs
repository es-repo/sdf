#[derive(Clone, Copy)]
pub struct DomainWarpingSceneControls {
    pub scale: f32,
    pub warp_strength: f32,
    pub amplitude: f32,
    pub gain: f32,
    pub octaves: u32,
    pub lacunarity: f32,
}

impl Default for DomainWarpingSceneControls {
    fn default() -> Self {
        Self {
            scale: 3.0,
            warp_strength: 0.05,
            amplitude: 0.5,
            gain: 0.5,
            octaves: 4,
            lacunarity: 2.0,
        }
    }
}

impl DomainWarpingSceneControls {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.scale, 0.1..=16.0).text("Scale"));
        ui.add(egui::Slider::new(&mut self.warp_strength, 0.0..=1.0).text("Strength"));
        ui.add(egui::Slider::new(&mut self.amplitude, 0.0..=2.0).text("Amplitude"));
        ui.add(egui::Slider::new(&mut self.gain, 0.0..=1.0).text("Gain"));
        ui.add(egui::Slider::new(&mut self.octaves, 1..=8).text("Octaves"));
        ui.add(egui::Slider::new(&mut self.lacunarity, 1.0..=4.0).text("Lacunarity"));

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}
