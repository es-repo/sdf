use super::Ray;
use crate::geometry::Vec3;
use pixels::wgpu::Color;

/// Distance-field result at a sampled point.
#[derive(Clone, Copy, Debug)]
pub struct SdfSample {
    pub dist: f32,
    pub color: Color,
}

impl SdfSample {
    pub fn new(dist: f32, color: Color) -> Self {
        Self { dist, color }
    }
}

/// Controls when ray marching stops.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct RayMarchSettings {
    pub max_steps: usize,
    pub hit_epsilon: f32,
    pub max_distance: f32,
}

/// First surface hit found by marching along a ray.
#[derive(Clone, Copy, Debug)]
pub struct RayMarchHit {
    pub point: Vec3,
    pub distance: f32,
    pub steps: usize,
    pub sample: SdfSample,
}

/// March result when the ray did not hit the sampled SDF.
#[derive(Clone, Copy, Debug)]
pub struct RayMarchMiss {
    pub distance: f32,
    pub steps: usize,
}

/// Result of marching a ray through an SDF scene.
#[derive(Clone, Copy, Debug)]
pub enum RayMarchResult {
    Hit(RayMarchHit),
    Miss(RayMarchMiss),
}

/// Marches along a ray until it hits the sampled SDF or exceeds the maximum distance.
pub fn ray_march<F>(ray: Ray, settings: RayMarchSettings, sample_sdf: F) -> RayMarchResult
where
    F: Fn(Vec3) -> SdfSample,
{
    let mut march_dist = 0.0;
    let mut steps = 0;

    for _ in 0..settings.max_steps {
        steps += 1;
        let point = ray.at(march_dist);
        let sample = sample_sdf(point);

        if sample.dist < settings.hit_epsilon {
            return RayMarchResult::Hit(RayMarchHit {
                point,
                distance: march_dist,
                steps,
                sample,
            });
        }

        march_dist += sample.dist.max(settings.hit_epsilon);

        if march_dist > settings.max_distance {
            break;
        }
    }

    RayMarchResult::Miss(RayMarchMiss {
        distance: march_dist,
        steps,
    })
}

/// Estimates an SDF surface normal using four tetrahedral samples.
pub fn estimate_normal_tetrahedral<F>(p: Vec3, e: f32, sdf: F) -> Vec3
where
    F: Fn(Vec3) -> f32,
{
    let k1 = Vec3::new(1.0, -1.0, -1.0);
    let k2 = Vec3::new(-1.0, -1.0, 1.0);
    let k3 = Vec3::new(-1.0, 1.0, -1.0);
    let k4 = Vec3::new(1.0, 1.0, 1.0);

    (k1 * sdf(p + k1 * e) + k2 * sdf(p + k2 * e) + k3 * sdf(p + k3 * e) + k4 * sdf(p + k4 * e)).normalize()
}

/// Estimates an SDF surface normal using central differences on each axis.
pub fn estimate_normal_central_differences<F>(p: Vec3, e: f32, sdf: F) -> Vec3
where
    F: Fn(Vec3) -> f32,
{
    Vec3::new(
        sdf(p + Vec3::new(e, 0.0, 0.0)) - sdf(p - Vec3::new(e, 0.0, 0.0)),
        sdf(p + Vec3::new(0.0, e, 0.0)) - sdf(p - Vec3::new(0.0, e, 0.0)),
        sdf(p + Vec3::new(0.0, 0.0, e)) - sdf(p - Vec3::new(0.0, 0.0, e)),
    )
    .normalize()
}
