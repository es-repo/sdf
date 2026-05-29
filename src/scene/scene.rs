use super::SceneFrame;
use crate::audio::AudioAnalysis;
use crate::input::InputState;

pub trait Scene: Send + Sync {
    fn update(&mut self, _delta_time: f32, _input: &InputState) {}

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

    fn save_state(&self) -> Option<serde_json::Value> {
        None
    }

    fn load_state(&mut self, _state: &serde_json::Value) {}
}
