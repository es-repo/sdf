use crate::color_ext::ColorExt;
use pixels::wgpu::Color;

#[derive(Clone, Copy, Debug)]
pub struct ExponentialFog {
    color: Color,
    density: f32,
}

impl ExponentialFog {
    pub fn new(color: Color, density: f32) -> Self {
        assert!(density.is_finite() && density >= 0.0);

        Self { color, density }
    }

    pub fn apply(&self, color: Color, distance: f32) -> Color {
        let fog_t = 1.0 - (-self.density * distance.max(0.0)).exp();

        color.lerp(self.color, fog_t)
    }
}
