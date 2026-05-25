use crate::geometry::Vec3;

pub trait Sdf3d {
    fn dist(&self, v: &Vec3) -> f32;

    fn dist_round(&self, v: &Vec3, r: f32) -> f32 {
        self.dist(v) - r
    }
}
