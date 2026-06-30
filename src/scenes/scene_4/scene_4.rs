use super::scene_4_controls::Scene4Controls;
use crate::audio::AudioAnalysis;
use crate::color_ext::ColorExt;
use crate::geometry::{Circle, Rectangle, Triangle};
use crate::geometry::{SignedDistance2d as _, Vec2};
use crate::math::smoothstep;
use crate::procedural::smooth_combine::smooth_union_color;
use crate::scene::{Scene, SceneFrame};
use pixels::wgpu::Color;

const AUDIO_TRACK: &str = "assets/audio/shadertoy_track1.mp3";

#[derive(Default)]
pub struct Scene4 {
    controls: Scene4Controls,
}

struct Scene4Frame {
    circle: Circle,
    rectangle: Rectangle,
    triangle: Triangle,
    round_radius: f32,
    smooth_blend_radius: f32,
}

impl SceneFrame for Scene4Frame {
    fn get_pixel_color(&self, coord: Vec2, scene_time: f32) -> Color {
        let c_dist = self.circle.dist_round(coord, self.round_radius);
        let r_dist = self.rectangle.dist_round(coord, self.round_radius);
        let t_dist = self.triangle.dist_round(coord, self.round_radius);

        let (dist, color) = smooth_union_color(
            c_dist,
            self.circle.color,
            r_dist,
            self.rectangle.color,
            self.smooth_blend_radius,
        );

        let (dist, color) = smooth_union_color(dist, color, t_dist, self.triangle.color, self.smooth_blend_radius);

        if dist < 0.0 {
            let c = Color::rgb(
                0.5 + (dist * 250.0 + scene_time).sin(),
                0.5 + (dist * 250.0).sin(),
                0.5 + (dist * 250.0).sin(),
            );
            let t = dist.abs() / self.circle.radius;
            color.lerp(c, t)
        } else if dist < 0.02 {
            Color::rgb(1.0, 1.0, 1.0)
        } else {
            let c = Color::rgb(0.1, 0.1, (dist * 150.0).sin().exp());
            let t = dist;
            Color::BLACK.lerp(c, t)
        }
    }
}

impl Scene for Scene4 {
    fn prepare_frame(&self, scene_time: f32) -> Box<dyn SceneFrame> {
        self.prepare_frame_with_audio(scene_time, &AudioAnalysis::default())
    }

    fn prepare_frame_with_audio(&self, scene_time: f32, audio: &AudioAnalysis) -> Box<dyn SceneFrame> {
        let time_scaled = scene_time * 0.5;
        let bass = audio.bass * 2.5 * self.controls.volume;
        let bass = if bass < 0.0001 { 1.0 } else { bass };
        let beat = smoothstep(0.1, 1.5, bass);
        let triangle_center = Vec2::new(time_scaled.cos(), 0.3 * time_scaled.sin());

        Box::new(Scene4Frame {
            circle: Circle {
                radius: 0.05 + 0.2 * bass,
                center: Vec2::new(
                    0.8 * (time_scaled + 0.2 + beat.sin()).cos(),
                    0.8 * (scene_time + beat.cos()).sin(),
                ),
                color: Color::rgb(0.7, 1.0, 0.0),
            },

            rectangle: Rectangle {
                center: Vec2::new(0.8 * scene_time.sin() * beat.sin(), 0.8 * (time_scaled + beat).cos()),
                vertex: Vec2::new(0.3, 0.2) * beat + 0.05,
                rotation: (time_scaled + beat).cos() * 0.5 * beat,
                color: Color::rgb(1.0, 0.7, 0.0),
            },

            triangle: Triangle {
                p0: (triangle_center + 0.05 + Vec2::new(-0.3, -0.15) * bass).rotate((scene_time + beat).sin()),
                p1: (triangle_center + 0.05 + Vec2::new(0.3, -0.15) * bass).rotate((time_scaled + beat).cos()),
                p2: (triangle_center + 0.05 + Vec2::new(0.0, 0.4) * bass).rotate(scene_time.sin()),
                color: Color::rgb(1.0, 0.0, 0.7),
            },

            round_radius: (0.5 + 0.5 * scene_time.sin()) * 0.1 * beat,
            smooth_blend_radius: 0.1 + (0.5 + 0.5 * scene_time.sin()) * 0.1 * beat,
        })
    }

    fn audio_track(&self) -> Option<&'static str> {
        Some(AUDIO_TRACK)
    }

    fn audio_volume(&self) -> f32 {
        self.controls.volume
    }

    fn has_controls_ui(&self) -> bool {
        true
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui) {
        self.controls.ui(ui);
    }
}
