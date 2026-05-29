use crate::geometry::{Quat, Vec3};

/// Camera orientation expressed as three orthogonal world-space directions.
///
/// `forward` is the direction the camera looks, `right` is the horizontal
/// screen direction, and `up` is the vertical screen direction. The default
/// basis looks along positive Z, with positive X to the right and positive Y up.
#[derive(Clone, Copy, Debug)]
pub struct CameraBasis {
    /// Direction from the camera into the scene.
    pub forward: Vec3,

    /// Horizontal screen direction, pointing to the camera's right.
    pub right: Vec3,

    /// Vertical screen direction, pointing to the camera's top.
    pub up: Vec3,
}

impl CameraBasis {
    /// Default camera basis: forward is +Z, right is +X, and up is +Y.
    pub const DEFAULT: Self = Self {
        forward: Vec3 { x: 0.0, y: 0.0, z: 1.0 },
        right: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
        up: Vec3 { x: 0.0, y: 1.0, z: 0.0 },
    };

    /// Returns this basis rotated by the given quaternion.
    pub fn rotated(self, rotation: Quat) -> Self {
        Self {
            forward: rotation * self.forward,
            right: rotation * self.right,
            up: rotation * self.up,
        }
    }
}
