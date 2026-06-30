use super::{Sdf, Vec2};
use pixels::wgpu::Color;

#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    pub p0: Vec2,
    pub p1: Vec2,
    pub p2: Vec2,
    pub color: Color,
}

impl Sdf for Triangle {
    fn dist(&self, point: Vec2) -> f32 {
        let e0 = self.p1 - self.p0;
        let e1 = self.p2 - self.p1;
        let e2 = self.p0 - self.p2;

        let v0 = point - self.p0;
        let v1 = point - self.p1;
        let v2 = point - self.p2;

        let pq0 = closest_segment_delta(v0, e0);
        let pq1 = closest_segment_delta(v1, e1);
        let pq2 = closest_segment_delta(v2, e2);

        let s = e0.cross(e2).signum();

        let d0 = Vec2::new(pq0.dot(pq0), s * v0.cross(e0));
        let d1 = Vec2::new(pq1.dot(pq1), s * v1.cross(e1));
        let d2 = Vec2::new(pq2.dot(pq2), s * v2.cross(e2));

        let d = d0.min(d1).min(d2);

        -d.x.sqrt() * d.y.signum()
    }
}

fn closest_segment_delta(v: Vec2, edge: Vec2) -> Vec2 {
    let t = (v.dot(edge) / edge.dot(edge)).clamp(0.0, 1.0);
    v - edge * t
}
