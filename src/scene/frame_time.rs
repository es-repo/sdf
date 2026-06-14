#[derive(Clone, Copy, Debug)]
pub struct FrameTime {
    /// Wall-clock time elapsed since the previous frame, in seconds.
    ///
    /// This keeps advancing while scene time is paused.
    pub real_time_delta: f32,

    /// Scene time elapsed since the previous frame, in seconds.
    ///
    /// This is `0.0` while scene time is paused.
    pub scene_time_delta: f32,

    /// Accumulated pausable scene time, in seconds.
    pub scene_time: f32,
}
