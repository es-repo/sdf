use crate::geometry::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: Vec3<f32>,
    pub direction: Vec3<f32>,
}

impl Ray {
    pub fn new(origin: Vec3<f32>, direction: Vec3<f32>) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, distance: f32) -> Vec3<f32> {
        self.origin + self.direction * distance
    }
}
