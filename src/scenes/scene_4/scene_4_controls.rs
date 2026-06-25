#[derive(Clone, Copy)]
pub struct Scene4Controls {
    pub volume: f32,
}

impl Default for Scene4Controls {
    fn default() -> Self {
        Self { volume: 1.0 }
    }
}

impl Scene4Controls {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}
