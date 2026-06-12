#[derive(Clone, Copy)]
pub struct DomainRepetitionSceneControls {
    pub spacing: f32,
}

impl Default for DomainRepetitionSceneControls {
    fn default() -> Self {
        Self { spacing: 5.0 }
    }
}

impl DomainRepetitionSceneControls {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.spacing, 1.2..=12.0).text("Spacing"));

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}
