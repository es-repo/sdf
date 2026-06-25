use crate::color_ext::ColorExt;
use pixels::wgpu::Color;

#[derive(Clone, Copy)]
pub struct Scene6Controls {
    pub object_color: Color,
    pub ambient_color: Color,
    pub step_blend_color: Color,
    pub step_blend_threshold: f32,
}

impl Default for Scene6Controls {
    fn default() -> Self {
        Self {
            object_color: Color::rgb(0.2, 0.13, 0.3),
            ambient_color: Color::rgb(0.7, 0.7, 1.0),
            step_blend_color: Color::WHITE,
            step_blend_threshold: 0.15,
        }
    }
}

impl Scene6Controls {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let mut rgb = [
                self.object_color.r as f32,
                self.object_color.g as f32,
                self.object_color.b as f32,
            ];

            if ui.color_edit_button_rgb(&mut rgb).changed() {
                self.object_color = Color::rgb(rgb[0], rgb[1], rgb[2]);
            }

            ui.label("Object color");
        });

        ui.horizontal(|ui| {
            let mut rgb = [
                self.ambient_color.r as f32,
                self.ambient_color.g as f32,
                self.ambient_color.b as f32,
            ];

            if ui.color_edit_button_rgb(&mut rgb).changed() {
                self.ambient_color = Color::rgb(rgb[0], rgb[1], rgb[2]);
            }

            ui.label("Ambient color");
        });

        ui.horizontal(|ui| {
            let mut rgb = [
                self.step_blend_color.r as f32,
                self.step_blend_color.g as f32,
                self.step_blend_color.b as f32,
            ];

            if ui.color_edit_button_rgb(&mut rgb).changed() {
                self.step_blend_color = Color::rgb(rgb[0], rgb[1], rgb[2]);
            }

            ui.label("Step blend color");
        });

        ui.add(egui::Slider::new(&mut self.step_blend_threshold, 0.01..=1.0).text("Step blend threshold"));

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}
