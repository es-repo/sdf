use super::{FrameTime, SceneFrame};
use crate::audio::AudioAnalysis;
use crate::input::InputState;

pub trait Scene: Send + Sync {
    /// Updates mutable scene state before a frame is prepared.
    ///
    /// `time` contains the real-time and scene-time values for the current
    /// frame. `input` is the current viewer input snapshot.
    fn update(&mut self, _time: FrameTime, _input: &InputState) {}

    /// Builds the immutable frame object used by the pixel renderer.
    ///
    /// `scene_time` is the accumulated pausable scene time in seconds.
    ///
    /// The returned frame should contain all data needed to render pixels
    /// without mutating the scene.
    fn prepare_frame(&self, scene_time: f32) -> Box<dyn SceneFrame>;

    /// Builds a frame using audio analysis data.
    ///
    /// Scenes that do not react to audio use the regular `prepare_frame`
    /// implementation.
    fn prepare_frame_with_audio(&self, scene_time: f32, _audio: &AudioAnalysis) -> Box<dyn SceneFrame> {
        self.prepare_frame(scene_time)
    }

    /// Returns true when this scene exposes editable controls through egui.
    fn has_controls_ui(&self) -> bool {
        false
    }

    /// Draws this scene's control widgets.
    ///
    /// This is called only when `has_controls_ui` returns true.
    fn controls_ui(&mut self, _ui: &mut egui::Ui) {}

    /// Returns an optional audio track path used by this scene.
    fn audio_track(&self) -> Option<&'static str> {
        None
    }

    /// Returns the playback volume for the active scene audio track.
    fn audio_volume(&self) -> f32 {
        1.0
    }

    /// Serializes scene-owned persistent state.
    ///
    /// Stateless scenes use the default `None`.
    fn save_state(&self) -> Option<serde_json::Value> {
        None
    }

    /// Restores scene-owned persistent state.
    ///
    /// Implementations should ignore incompatible values.
    fn load_state(&mut self, _state: &serde_json::Value) {}
}
