use crate::color_ext::ColorExt;
use pixels::wgpu::Color;

#[derive(Clone, Copy, Debug)]
pub struct ExponentialFog {
    color: Color,
    density: f32,
    start_distance: f32,
}

impl ExponentialFog {
    pub fn new(color: Color, density: f32, start_distance: f32) -> Self {
        assert!(density.is_finite() && density >= 0.0);
        assert!(start_distance.is_finite() && start_distance >= 0.0);

        Self {
            color,
            density,
            start_distance,
        }
    }

    pub fn apply(&self, color: Color, distance: f32) -> Color {
        let fog_distance = (distance - self.start_distance).max(0.0);
        let fog_t = 1.0 - (-self.density * fog_distance).exp();

        color.lerp(self.color, fog_t)
    }
}
