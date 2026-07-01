use crate::rendering::RayMarchSettings;

pub fn ray_march_settings_ui(ui: &mut egui::Ui, settings: &mut RayMarchSettings) {
    ui.add(egui::Slider::new(&mut settings.max_steps, 1..=512).text("Max steps"));
    ui.add(
        egui::Slider::new(&mut settings.hit_epsilon, 0.00001..=0.01)
            .logarithmic(true)
            .text("Hit epsilon"),
    );
    ui.add(egui::Slider::new(&mut settings.max_distance, 1.0..=500.0).text("Max distance"));
    ui.add(
        egui::Slider::new(&mut settings.min_step, 0.0001..=0.05)
            .logarithmic(true)
            .text("Min step"),
    );
    ui.add(egui::Slider::new(&mut settings.near_clip, 0.0..=1.0).text("Near clip"));
}
