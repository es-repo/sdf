use pixels::wgpu::Color;

use crate::color_ext::ColorExt;
use crate::geometry::Vec2;
use crate::procedural::{Fbm, NoiseSimplex};
use crate::scene::{Scene, SceneFrame};

pub struct SimplexNoiseScene;

struct SimplexNoiseSceneFrame {}

impl Scene for SimplexNoiseScene {
    fn prepare_frame(&self, _scene_time: f32) -> Box<dyn SceneFrame> {
        Box::new(SimplexNoiseSceneFrame {})
    }
}

impl SceneFrame for SimplexNoiseSceneFrame {
    fn get_pixel_color(&self, coord: Vec2, scene_time: f32) -> Color {
        let mut f;
        let scale = 3.0;
        let time_scaled = scene_time * 0.2;

        if coord.x < 0.0 && coord.y > 0.0 {
            let coord = coord * scale + time_scaled;
            f = coord.noise_simplex();
        } else {
            let octaves = if coord.x > 0.0 && coord.y > 0.0 {
                2
            } else if coord.x < 0.0 && coord.y < 0.0 {
                3
            } else {
                4
            };

            let coord = coord * scale + time_scaled;
            f = coord.fbm_rotated(octaves, 0.5, 0.5, |coord| coord.noise_simplex());
        }

        f = 0.5 + 0.5 * f;

        Color::rgb(f, f, f)
    }
}
