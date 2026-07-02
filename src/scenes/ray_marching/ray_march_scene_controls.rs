use crate::rendering::RayMarchSettings;

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(super) struct RayMarchSceneControls {
    pub ray_march_settings: RayMarchSettings,
    pub animate_ground: bool,
    pub show_convergence_failure_debug: bool,
}

impl Default for RayMarchSceneControls {
    fn default() -> Self {
        Self {
            ray_march_settings: RayMarchSettings {
                max_steps: 400,
                hit_epsilon: 0.001,
                max_distance: 100.0,
                min_step: 0.005,
                near_clip: 0.05,
            },
            animate_ground: false,
            show_convergence_failure_debug: false,
        }
    }
}

impl RayMarchSceneControls {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.ray_march_settings.max_steps, 1..=512).text("Max steps"));
        ui.add(
            egui::Slider::new(&mut self.ray_march_settings.hit_epsilon, 0.00001..=0.01)
                .logarithmic(true)
                .text("Hit epsilon"),
        );
        ui.add(egui::Slider::new(&mut self.ray_march_settings.max_distance, 1.0..=500.0).text("Max distance"));
        ui.add(
            egui::Slider::new(&mut self.ray_march_settings.min_step, 0.0001..=0.05)
                .logarithmic(true)
                .text("Min step"),
        );
        ui.add(egui::Slider::new(&mut self.ray_march_settings.near_clip, 0.0..=1.0).text("Near clip"));
        ui.checkbox(&mut self.animate_ground, "Animate ground");
        ui.checkbox(
            &mut self.show_convergence_failure_debug,
            "Show convergence failure debug",
        );

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}
