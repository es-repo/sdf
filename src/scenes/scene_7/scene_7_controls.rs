use crate::color_ext::ColorExt;
use pixels::wgpu::Color;

#[derive(Clone, Copy)]
pub struct Scene7Controls {
    pub object_color: Color,
    pub out_color: Color,
    pub in_color: Color,
    pub spheres_per_ring: usize,
}

impl Default for Scene7Controls {
    fn default() -> Self {
        Self {
            object_color: Color::rgb(0.5, 0.3, 1.0),
            out_color: Color::rgb(0.1, 1.0, 0.0),
            in_color: Color::rgb(1.0, 0.0, 0.0),
            spheres_per_ring: 8,
        }
    }
}

impl Scene7Controls {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        color_control(ui, &mut self.object_color, "Color");
        color_control(ui, &mut self.out_color, "Out color");
        color_control(ui, &mut self.in_color, "In color");

        ui.add(egui::Slider::new(&mut self.spheres_per_ring, 2..=16).text("Count"));

        if ui.button("Reset").clicked() {
            *self = Self::default();
        }
    }
}

fn color_control(ui: &mut egui::Ui, color: &mut Color, label: &str) {
    ui.horizontal(|ui| {
        let mut rgb = [color.r as f32, color.g as f32, color.b as f32];

        if ui.color_edit_button_rgb(&mut rgb).changed() {
            *color = Color::rgb(rgb[0], rgb[1], rgb[2]);
        }

        ui.label(label);
    });
}
