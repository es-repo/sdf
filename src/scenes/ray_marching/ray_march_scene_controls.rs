use crate::rendering::RayMarchSettings;

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(super) struct RayMarchSceneControls {
    pub ray_march_settings: RayMarchSettings,
    #[serde(default = "default_shadow_softness")]
    pub shadow_softness: f32,
    pub animate_ground: bool,
    pub show_convergence_failure_debug: bool,
}

fn default_shadow_softness() -> f32 {
    0.05
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
            shadow_softness: default_shadow_softness(),
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
        ui.add(egui::Slider::new(&mut self.shadow_softness, 0.0..=0.5).text("Shadow softness"));
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
