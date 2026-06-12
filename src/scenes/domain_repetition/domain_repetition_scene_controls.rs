#[derive(Clone, Copy)]
pub struct DomainRepetitionSceneControls {
    pub spacing: f32,
    pub max_distance: f32,
}

impl Default for DomainRepetitionSceneControls {
    fn default() -> Self {
        Self {
            spacing: 5.0,
            max_distance: 100.0,
        }
    }
}

impl DomainRepetitionSceneControls {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.spacing, 1.2..=12.0).text("Spacing"));
        ui.add(egui::Slider::new(&mut self.max_distance, 10.0..=500.0).text("Max distance"));

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}
