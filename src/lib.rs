pub mod color_ext;
pub mod geometry;
pub mod math;
pub mod procedural;
pub mod scenes;

pub use color_ext::ColorExt;
pub use geometry::{Circle, Sdf, Vec2, Vec3};
pub use math::{floor_i32, lerp, max_pair, min_pair, unlerp, unlerp_unit};
pub use procedural::{smooth_union, Fbm, NoiseSimplex};
