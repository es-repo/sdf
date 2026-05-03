use super::Vec2;

pub trait Sdf {
    fn dist(&self, v: &Vec2<f32>) -> f32;
}
