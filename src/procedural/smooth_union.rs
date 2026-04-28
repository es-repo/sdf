use crate::lerp;

pub fn smooth_union(d1: f32, d2: f32, k: f32) -> (f32, f32) {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    let d = lerp(d2, d1, h) - k * h * (1.0 - h);
    (d, h)
}
