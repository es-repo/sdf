use crate::audio::AudioAnalysis;
use crate::geometry::Vec2;
use pixels::wgpu::Color;

pub trait SceneFrame: Send + Sync {
    fn get_pixel_color(&self, coord: Vec2, time: f32) -> Color;

    fn get_pixel_color_with_audio(&self, coord: Vec2, time: f32, _audio: &AudioAnalysis) -> Color {
        self.get_pixel_color(coord, time)
    }
}

pub trait Scene: Send + Sync {
    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame>;

    fn prepare_frame_with_audio(&self, time: f32, _audio: &AudioAnalysis) -> Box<dyn SceneFrame> {
        self.prepare_frame(time)
    }

    fn audio_track(&self) -> Option<&'static str> {
        None
    }

    fn audio_volume(&self) -> f32 {
        1.0
    }
}

pub trait ParameterizedScene: Scene {
    fn parameters_ui(&mut self, ui: &mut egui::Ui);
}

pub enum SceneInstance {
    Plain(Box<dyn Scene>),
    Parameterized(Box<dyn ParameterizedScene>),
}

impl SceneInstance {
    pub fn plain<T>(scene: T) -> Self
    where
        T: Scene + 'static,
    {
        Self::Plain(Box::new(scene))
    }

    pub fn parameterized<T>(scene: T) -> Self
    where
        T: ParameterizedScene + 'static,
    {
        Self::Parameterized(Box::new(scene))
    }

    pub fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        match self {
            Self::Plain(scene) => scene.prepare_frame(time),
            Self::Parameterized(scene) => scene.prepare_frame(time),
        }
    }

    pub fn prepare_frame_with_audio(&self, time: f32, audio: &AudioAnalysis) -> Box<dyn SceneFrame> {
        match self {
            Self::Plain(scene) => scene.prepare_frame_with_audio(time, audio),
            Self::Parameterized(scene) => scene.prepare_frame_with_audio(time, audio),
        }
    }

    pub fn audio_track(&self) -> Option<&'static str> {
        match self {
            Self::Plain(scene) => scene.audio_track(),
            Self::Parameterized(scene) => scene.audio_track(),
        }
    }

    pub fn audio_volume(&self) -> f32 {
        match self {
            Self::Plain(scene) => scene.audio_volume(),
            Self::Parameterized(scene) => scene.audio_volume(),
        }
    }

    pub fn parameterized_scene(&self) -> Option<&dyn ParameterizedScene> {
        match self {
            Self::Plain(_) => None,
            Self::Parameterized(scene) => Some(scene.as_ref()),
        }
    }

    pub fn parameterized_scene_mut(&mut self) -> Option<&mut dyn ParameterizedScene> {
        match self {
            Self::Plain(_) => None,
            Self::Parameterized(scene) => Some(scene.as_mut()),
        }
    }
}
