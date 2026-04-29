use crate::scenes::{ParameterizedScene, Scene, SceneFrame};
use crate::{Circle, Fbm, NoiseSimplex, Vec2};
use pixels::wgpu::Color;

#[derive(Clone, Copy)]
pub struct DomainWarpingParams {
    pub scale: f32,
    pub warp_strength: f32,
    pub octaves: u32,
    pub lacunarity: f32,
}

impl Default for DomainWarpingParams {
    fn default() -> Self {
        Self {
            scale: 3.0,
            warp_strength: 0.05,
            octaves: 4,
            lacunarity: 2.0,
        }
    }
}

#[derive(Default)]
pub struct DomainWarping {
    params: DomainWarpingParams,
}

struct DomainWarpingFrame {
    circle: Circle,
    params: DomainWarpingParams,
    time_scaled: f32,
}

impl Scene for DomainWarping {
    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        let circle = Circle {
            center: Vec2::new(0.0, 0.0),
            radius: 0.5,
            color: Color {
                r: 0.3,
                g: 1.0,
                b: 0.4,
                a: 1.0,
            },
        };

        Box::new(DomainWarpingFrame {
            circle,
            params: self.params,
            time_scaled: time * 0.25,
        })
    }
}

impl ParameterizedScene for DomainWarping {
    fn parameters_ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.params.scale, 0.1..=16.0).text("Scale"));
        ui.add(egui::Slider::new(&mut self.params.warp_strength, 0.0..=1.0).text("Strength"));
        ui.add(egui::Slider::new(&mut self.params.octaves, 1..=8).text("Octaves"));
        ui.add(egui::Slider::new(&mut self.params.lacunarity, 1.0..=4.0).text("Lacunarity"));

        if ui.button("Reset").clicked() {
            self.params = DomainWarpingParams::default();
        }
    }
}

impl SceneFrame for DomainWarpingFrame {
    fn get_pixel_color(&self, coord: Vec2<f32>, _time: f32) -> Color {
        let noise_coord = coord * self.params.scale + self.time_scaled;
        let offset = noise_coord.fbm(self.params.octaves, 0.5, 0.5, self.params.lacunarity, |coord| {
            coord.noise_simplex()
        }) * self.params.warp_strength;

        let warped_coord = coord + offset;

        let dist = self.circle.dist(&warped_coord);

        if dist < 0.0 { self.circle.color } else { Color::BLACK }
    }
}
