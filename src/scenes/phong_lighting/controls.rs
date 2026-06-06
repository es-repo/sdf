use crate::color_ext::ColorExt;
use pixels::wgpu::Color;

#[derive(Clone, Copy)]
pub struct PhongLightingSceneControls {
    pub light_color: Color,
    pub light_intensity: f32,
    pub ambient_color: Color,
    pub ambient_intensity: f32,
    pub object_color: Color,
    pub specular_color: Color,
    pub specular_shininess: f32,
}

impl Default for PhongLightingSceneControls {
    fn default() -> Self {
        Self {
            light_color: Color::WHITE,
            light_intensity: 1.0,
            ambient_color: Color::rgb(0.7, 0.7, 1.0),
            ambient_intensity: 0.75,
            object_color: Color::rgb(1.0, 0.5, 0.0),
            specular_color: Color::WHITE,
            specular_shininess: 50.0,
        }
    }
}

impl PhongLightingSceneControls {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        color_control(ui, "Object color", &mut self.object_color);
        ui.separator();

        color_control(ui, "Light color", &mut self.light_color);
        ui.add(egui::Slider::new(&mut self.light_intensity, 0.0..=5.0).text("Intensity"));

        ui.separator();

        color_control(ui, "Ambient color", &mut self.ambient_color);
        ui.add(egui::Slider::new(&mut self.ambient_intensity, 0.0..=2.0).text("Intensity"));

        ui.separator();

        color_control(ui, "Specular color", &mut self.specular_color);
        ui.add(egui::Slider::new(&mut self.specular_shininess, 1.0..=128.0).text("Shininess"));

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}

fn color_control(ui: &mut egui::Ui, label: &str, color: &mut Color) {
    ui.horizontal(|ui| {
        let mut rgb = [color.r as f32, color.g as f32, color.b as f32];

        if ui.color_edit_button_rgb(&mut rgb).changed() {
            *color = Color::rgb(rgb[0], rgb[1], rgb[2]);
        }

        ui.label(label);
    });
}
