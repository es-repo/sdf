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
    /// Maximum number of SDF samples before the ray is treated as a miss.
    pub max_steps: usize,

    /// Distance from the surface at which the ray is considered to have hit.
    pub hit_epsilon: f32,

    /// Maximum distance the ray can travel before it is treated as a miss.
    pub max_distance: f32,

    /// Smallest distance the ray advances on one step.
    pub min_step: f32,

    /// Initial distance from the ray origin before marching starts.
    ///
    /// This behaves like a camera near clipping plane and prevents immediate hits
    /// at the camera origin when the camera is very close to geometry.
    #[serde(default = "default_near_clip")]
    pub near_clip: f32,
}

fn default_near_clip() -> f32 {
    0.05
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
    let mut march_dist = settings.near_clip.max(0.0);
    let mut steps = 0;

    if march_dist > settings.max_distance {
        return RayMarchResult::Miss(RayMarchMiss {
            distance: march_dist,
            steps,
        });
    }

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

        march_dist += sample.dist.max(settings.min_step);

        if march_dist > settings.max_distance {
            break;
        }
    }

    RayMarchResult::Miss(RayMarchMiss {
        distance: march_dist,
        steps,
    })
}

/// Computes shadow visibility for a surface point and a point light.
///
/// A secondary ray is marched from the surface toward `light_position`. The
/// `sample_sdf` callback must return the signed distance to the closest scene
/// surface. An actual hit returns `0.0`. Otherwise, the returned visibility is
/// between `0.0` and `1.0` based on how closely the ray passes nearby geometry.
/// Geometry beyond the light cannot cast a shadow on the surface point.
///
/// A `softness` of `0.0` produces hard shadows. Increasing it creates a wider
/// penumbra by reducing visibility where the ray passes close to geometry.
///
/// The ray origin is offset along `surface_normal` to avoid immediately hitting
/// the surface that produced the primary ray hit. The bias accounts for both
/// `hit_epsilon` and possible overshoot caused by `min_step`. If `max_steps` is
/// exhausted, the best visibility estimate collected so far is returned. The
/// result is intended to scale direct diffuse and specular lighting while leaving
/// ambient lighting unchanged.
pub fn shadow<F>(
    surface_point: Vec3,
    surface_normal: Vec3,
    light_position: Vec3,
    settings: RayMarchSettings,
    softness: f32,
    sample_sdf: F,
) -> f32
where
    F: Fn(Vec3) -> f32,
{
    // The primary march may overshoot the surface by up to `min_step`.
    let shadow_bias = settings.hit_epsilon.max(settings.min_step) * 2.0;
    let shadow_origin = surface_point + surface_normal * shadow_bias;
    let to_light = light_position - shadow_origin;
    let light_distance = to_light.len();

    if light_distance <= settings.hit_epsilon {
        return 1.0;
    }

    let shadow_ray = Ray::new(shadow_origin, to_light / light_distance);
    let mut march_dist = 0.0;
    let mut visibility: f32 = 1.0;
    let softness = softness.max(0.0);

    for _ in 0..settings.max_steps {
        if march_dist >= light_distance {
            return visibility.clamp(0.0, 1.0);
        }

        let dist = sample_sdf(shadow_ray.at(march_dist));

        if dist < settings.hit_epsilon {
            return 0.0;
        }

        if softness > 0.0 && march_dist > 0.0 {
            visibility = visibility.min(dist / (softness * march_dist));
        }

        march_dist += dist.max(settings.min_step);
    }

    visibility.clamp(0.0, 1.0)
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
