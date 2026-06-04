use crate::math::lerp;
use pixels::wgpu::Color;

/// Convenience operations for `wgpu::Color`.
pub trait ColorExt {
    /// Creates an opaque color from normalized RGB components.
    fn rgb(r: f32, g: f32, b: f32) -> Self
    where
        Self: Sized,
    {
        Self::rgba(r, g, b, 1.0)
    }

    /// Creates a color from normalized RGBA components.
    fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self
    where
        Self: Sized;

    /// Converts normalized RGBA components to 8-bit RGBA components.
    fn to_u8_array(&self) -> [u8; 4];

    /// Alpha-composites this source color over the destination color.
    fn blend(&self, dst: Color) -> Color;

    /// Linearly interpolates all RGBA components toward another color.
    fn lerp(&self, other: Color, t: f32) -> Color;

    /// Linearly interpolates RGB components toward a gray value, preserving alpha.
    fn lerp_gray(&self, value: f32, t: f32) -> Color;

    /// Multiplies RGB components by a scalar value, preserving alpha.
    fn scale_rgb(&self, factor: f32) -> Color;

    /// Adds RGB components from another color, preserving this color's alpha.
    fn add_rgb(&self, other: Color) -> Color;

    /// Multiplies RGB components by another color's RGB components, preserving this color's alpha.
    fn multiply_rgb(&self, other: Color) -> Color;

    /// Returns this color with a replaced alpha component.
    fn with_alpha(&self, alpha: f64) -> Color;
}

impl ColorExt for Color {
    fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r as f64,
            g: g as f64,
            b: b as f64,
            a: a as f64,
        }
    }

    fn to_u8_array(&self) -> [u8; 4] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ]
    }

    fn blend(&self, dst: Color) -> Color {
        let sa = self.a.clamp(0.0, 1.0);
        let da = dst.a.clamp(0.0, 1.0);

        let out_a = sa + da * (1.0 - sa);

        if out_a == 0.0 {
            return Self::rgba(0.0, 0.0, 0.0, 0.0);
        }

        Self::rgba(
            ((self.r * sa + dst.r * da * (1.0 - sa)) / out_a) as f32,
            ((self.g * sa + dst.g * da * (1.0 - sa)) / out_a) as f32,
            ((self.b * sa + dst.b * da * (1.0 - sa)) / out_a) as f32,
            out_a as f32,
        )
    }

    fn lerp(&self, other: Color, t: f32) -> Color {
        Self::rgba(
            lerp(self.r, other.r, t as f64) as f32,
            lerp(self.g, other.g, t as f64) as f32,
            lerp(self.b, other.b, t as f64) as f32,
            lerp(self.a, other.a, t as f64) as f32,
        )
    }

    fn lerp_gray(&self, value: f32, t: f32) -> Color {
        self.lerp(Self::rgba(value, value, value, self.a as f32), t)
    }

    fn scale_rgb(&self, factor: f32) -> Color {
        let factor = factor as f64;

        Self::rgba(
            (self.r * factor) as f32,
            (self.g * factor) as f32,
            (self.b * factor) as f32,
            self.a as f32,
        )
    }

    fn add_rgb(&self, other: Color) -> Color {
        Self::rgba(
            (self.r + other.r) as f32,
            (self.g + other.g) as f32,
            (self.b + other.b) as f32,
            self.a as f32,
        )
    }

    fn multiply_rgb(&self, other: Color) -> Color {
        Self::rgba(
            (self.r * other.r) as f32,
            (self.g * other.g) as f32,
            (self.b * other.b) as f32,
            self.a as f32,
        )
    }

    fn with_alpha(&self, alpha: f64) -> Color {
        Self::rgba(self.r as f32, self.g as f32, self.b as f32, alpha as f32)
    }
}
