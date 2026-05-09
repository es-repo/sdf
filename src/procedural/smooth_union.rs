use crate::{ColorExt, lerp};
use pixels::wgpu::Color;

/// Blends two signed distances into a smooth union.
///
/// `d1` and `d2` are signed distances to the two shapes. `k` controls the
/// blend radius: larger values make the transition softer and wider.
///
/// Returns the blended signed distance and the interpolation factor used to
/// mix values associated with the two shapes.
pub fn smooth_union(d1: f32, d2: f32, k: f32) -> (f32, f32) {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    let d = lerp(d2, d1, h) - k * h * (1.0 - h);
    (d, h)
}

/// Blends two signed distances and interpolates their colors.
///
/// The color interpolation uses the same factor returned by `smooth_union`:
/// when the first distance dominates, the result moves toward `color1`; when
/// the second distance dominates, it moves toward `color2`.
pub fn smooth_union_color(d1: f32, color1: Color, d2: f32, color2: Color, k: f32) -> (f32, Color) {
    let (d, h) = smooth_union(d1, d2, k);
    (d, color2.lerp(color1, h))
}
