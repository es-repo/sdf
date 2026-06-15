use crate::color_ext::ColorExt;
use pixels::wgpu::Color;

#[derive(Clone, Copy, Debug)]
pub struct ExponentialFog {
    pub color: Color,
    pub density: f32,
}

impl ExponentialFog {
    pub fn apply(&self, color: Color, distance: f32) -> Color {
        let fog_t = 1.0 - (-self.density.max(0.0) * distance.max(0.0)).exp();

        color.lerp(self.color, fog_t)
    }
}
