use crate::geometry::Vec3;

pub trait SignedDistance3d {
    fn dist(&self, point: Vec3) -> f32;

    fn dist_round(&self, point: Vec3, radius: f32) -> f32 {
        self.dist(point) - radius
    }
}
