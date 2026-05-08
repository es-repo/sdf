use crate::lerp;

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
