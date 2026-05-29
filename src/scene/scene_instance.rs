use super::{ParameterizedScene, Scene, SceneFrame};
use crate::audio::AudioAnalysis;
use crate::input::InputState;

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

    pub fn update(&mut self, delta_time: f32, input: &InputState) {
        match self {
            Self::Plain(scene) => scene.update(delta_time, input),
            Self::Parameterized(scene) => scene.update(delta_time, input),
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

    pub fn save_state(&self) -> Option<serde_json::Value> {
        match self {
            Self::Plain(scene) => scene.save_state(),
            Self::Parameterized(scene) => scene.save_state(),
        }
    }

    pub fn load_state(&mut self, state: &serde_json::Value) {
        match self {
            Self::Plain(scene) => scene.load_state(state),
            Self::Parameterized(scene) => scene.load_state(state),
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
