#[derive(Clone, Copy)]
pub struct DomainRepetitionSceneControls {
    pub spacing: f32,
    pub max_distance: f32,
    pub fog_density: f32,
}

impl Default for DomainRepetitionSceneControls {
    fn default() -> Self {
        Self {
            spacing: 4.0,
            max_distance: 70.0,
            fog_density: 0.05,
        }
    }
}

impl DomainRepetitionSceneControls {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.spacing, 1.2..=16.0).text("Spacing"));
        ui.add(egui::Slider::new(&mut self.max_distance, 10.0..=200.0).text("Max distance"));
        ui.add(egui::Slider::new(&mut self.fog_density, 0.0..=0.9).text("Fog density"));

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}
