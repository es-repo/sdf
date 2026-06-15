#[derive(Clone, Copy)]
pub struct DomainRepetitionSceneControls {
    pub spacing: f32,
    pub max_distance: f32,
}

impl Default for DomainRepetitionSceneControls {
    fn default() -> Self {
        Self {
            spacing: 3.0,
            max_distance: 70.0,
        }
    }
}

impl DomainRepetitionSceneControls {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.spacing, 1.2..=16.0).text("Spacing"));
        ui.add(egui::Slider::new(&mut self.max_distance, 10.0..=200.0).text("Max distance"));

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}
