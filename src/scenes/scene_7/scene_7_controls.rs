use crate::color_ext::ColorExt;
use pixels::wgpu::Color;

#[derive(Clone, Copy)]
pub struct Scene7Controls {
    pub object_color: Color,
}

impl Default for Scene7Controls {
    fn default() -> Self {
        Self {
            object_color: Color::rgb(0.5, 0.3, 1.0),
        }
    }
}

impl Scene7Controls {
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

            ui.label("Color");
        });

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}
