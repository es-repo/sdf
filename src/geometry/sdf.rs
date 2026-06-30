use super::Vec2;

pub trait Sdf {
    fn dist(&self, point: Vec2) -> f32;

    fn dist_round(&self, point: Vec2, radius: f32) -> f32 {
        self.dist(point) - radius
    }
}
