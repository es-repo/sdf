use crate::color_ext::ColorExt;
use pixels::wgpu::Color;

#[derive(Clone, Copy)]
pub struct Scene5Controls {
    pub sphere_color: Color,
}

impl Default for Scene5Controls {
    fn default() -> Self {
        Self {
            sphere_color: Color::rgb(0.009125, 0.009125, 0.009125),
        }
    }
}

impl Scene5Controls {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let mut rgb = [
                self.sphere_color.r as f32,
                self.sphere_color.g as f32,
                self.sphere_color.b as f32,
            ];

            if ui.color_edit_button_rgb(&mut rgb).changed() {
                self.sphere_color = Color::rgb(rgb[0], rgb[1], rgb[2]);
            }

            ui.label("Color");
        });

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}
