use crate::noise_simplex::NoiseSimplex;
use crate::vec2::Vec2;
use crate::vec3::Vec3;
use std::ops::Mul;

pub trait Fbm {
    type VecN: Copy + Mul<f32, Output = Self::VecN>;

    fn fbm<N>(&self, octaves: u32, amplitude: f32, gain: f32, lacunarity: f32, noise: N) -> f32
    where
        N: Fn(Self::VecN) -> f32,
    {
        self.fbm_with_transform(octaves, amplitude, gain, noise, |coord| coord * lacunarity)
    }

    fn fbm_with_transform<N, T>(&self, octaves: u32, amplitude: f32, gain: f32, noise: N, transform: T) -> f32
    where
        N: Fn(Self::VecN) -> f32,
        T: Fn(Self::VecN) -> Self::VecN;

    fn fbm_rotated(&self, octaves: u32, amplitude: f32, gain: f32) -> f32;
}

impl Fbm for Vec2<f32> {
    type VecN = Vec2<f32>;

    fn fbm_with_transform<N, T>(&self, octaves: u32, amplitude: f32, gain: f32, noise: N, transform: T) -> f32
    where
        N: Fn(Self::VecN) -> f32,
        T: Fn(Self::VecN) -> Self::VecN,
    {
        let mut coord = *self;
        let mut value = 0.0;
        let mut amplitude = amplitude;

        for _ in 0..octaves {
            value += amplitude * noise(coord);
            coord = transform(coord);
            amplitude *= gain;
        }

        value
    }

    fn fbm_rotated(&self, octaves: u32, amplitude: f32, gain: f32) -> f32 {
        self.fbm_with_transform(
            octaves,
            amplitude,
            gain,
            |coord| coord.noise_simplex(),
            |coord| Vec2::new(1.6 * coord.x + 1.2 * coord.y, -1.2 * coord.x + 1.6 * coord.y),
        )
    }
}

impl Fbm for Vec3<f32> {
    type VecN = Vec3<f32>;

    fn fbm_with_transform<N, T>(&self, octaves: u32, amplitude: f32, gain: f32, noise: N, transform: T) -> f32
    where
        N: Fn(Self::VecN) -> f32,
        T: Fn(Self::VecN) -> Self::VecN,
    {
        let mut coord = *self;
        let mut value = 0.0;
        let mut amplitude = amplitude;

        for _ in 0..octaves {
            value += amplitude * noise(coord);
            coord = transform(coord);
            amplitude *= gain;
        }

        value
    }

    fn fbm_rotated(&self, octaves: u32, amplitude: f32, gain: f32) -> f32 {
        self.fbm_with_transform(
            octaves,
            amplitude,
            gain,
            |coord| coord.noise_simplex(),
            |coord| {
                Vec3::new(
                    1.6 * coord.x + 1.2 * coord.y + 0.8 * coord.z,
                    -1.2 * coord.x + 1.6 * coord.y - 0.8 * coord.z,
                    -0.8 * coord.x + 0.8 * coord.y + 1.6 * coord.z,
                )
            },
        )
    }
}
