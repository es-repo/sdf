pub mod color_ext;
pub mod scenes;
pub mod vec2;
pub mod vec3;

pub use color_ext::ColorExt;
pub use vec2::Vec2;
pub use vec3::Vec3;

pub fn smooth_union(d1: f32, d2: f32, k: f32) -> (f32, f32) {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    let d = d2 * (1.0 - h) + d1 * h - k * h * (1.0 - h);
    (d, h)
}
