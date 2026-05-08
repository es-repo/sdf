use super::Vec2;

pub trait Sdf {
    fn dist(&self, v: &Vec2<f32>) -> f32;

    fn dist_round(&self, v: &Vec2<f32>, r: f32) -> f32 {
        self.dist(v) - r
    }
}
