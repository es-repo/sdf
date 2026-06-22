use crate::geometry::AxisSet;

#[derive(Clone, Copy)]
pub struct DomainRepetitionSceneControls {
    pub spacing: f32,
    pub axes: AxisSet,
    pub max_distance: f32,
    pub fog_density: f32,
}

impl Default for DomainRepetitionSceneControls {
    fn default() -> Self {
        Self {
            spacing: 4.0,
            axes: AxisSet::XYZ,
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

        egui::ComboBox::from_label("Axes")
            .selected_text(self.axes.label())
            .show_ui(ui, |ui| {
                for axes in AxisSet::ALL {
                    ui.selectable_value(&mut self.axes, axes, axes.label());
                }
            });

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}
