use crate::color_ext::ColorExt;
use crate::math::lerp;
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

/// Smoothly unions multiple signed distances with the same blend radius.
///
/// Returns `None` when no distances are provided. For non-empty input, the
/// distances are folded pair by pair using [`smooth_union`].
pub fn smooth_union_many<I>(distances: I, k: f32) -> Option<f32>
where
    I: IntoIterator<Item = f32>,
{
    let mut distances = distances.into_iter();
    let first = distances.next()?;

    Some(distances.fold(first, |dist, next_dist| smooth_union(dist, next_dist, k).0))
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

/// Blends two signed distances into a smooth intersection.
///
/// `d1` and `d2` are signed distances to the two shapes. `k` controls the
/// blend radius: larger values make the intersection boundary softer and wider.
///
/// Returns the blended signed distance and the interpolation factor used to
/// mix values associated with the two shapes.
pub fn smooth_intersection(d1: f32, d2: f32, k: f32) -> (f32, f32) {
    let h = (0.5 - 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    let d = lerp(d2, d1, h) + k * h * (1.0 - h);
    (d, h)
}

/// Blends two signed distances into a smooth intersection and interpolates their colors.
///
/// The color interpolation uses the same factor returned by `smooth_intersection`:
/// when the first distance dominates, the result moves toward `color1`; when
/// the second distance dominates, it moves toward `color2`.
pub fn smooth_intersection_color(d1: f32, color1: Color, d2: f32, color2: Color, k: f32) -> (f32, Color) {
    let (d, h) = smooth_intersection(d1, d2, k);
    (d, color2.lerp(color1, h))
}

/// Smoothly subtracts the second signed distance field from the first.
///
/// `d1` is the shape to keep, `d2` is the shape to cut away, and `k` controls
/// the blend radius around the cut.
pub fn smooth_subtraction(d1: f32, d2: f32, k: f32) -> (f32, f32) {
    smooth_intersection(d1, -d2, k)
}

/// Smoothly subtracts multiple signed distances from the first one.
///
/// Returns `None` when no distances are provided. For non-empty input, the
/// first distance is retained and each remaining distance is subtracted from
/// the accumulated result using [`smooth_subtraction`].
pub fn smooth_subtraction_many<I>(distances: I, k: f32) -> Option<f32>
where
    I: IntoIterator<Item = f32>,
{
    let mut distances = distances.into_iter();
    let first = distances.next()?;

    Some(distances.fold(first, |dist, next_dist| smooth_subtraction(dist, next_dist, k).0))
}

/// Smoothly subtracts the second signed distance field from the first and interpolates colors.
///
/// `color1` is the color of the kept shape. `color2` can be used to tint the
/// smooth cut region; pass `color1` as both colors when the subtraction should
/// keep a single material color.
pub fn smooth_subtraction_color(d1: f32, color1: Color, d2: f32, color2: Color, k: f32) -> (f32, Color) {
    let (d, h) = smooth_subtraction(d1, d2, k);
    (d, color2.lerp(color1, h))
}
