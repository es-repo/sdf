mod analysis;
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use analysis::{AUDIO_SPECTRUM_BINS, AudioAnalysis};
#[cfg(not(target_arch = "wasm32"))]
pub use native::AudioTrack;
#[cfg(target_arch = "wasm32")]
pub use wasm::AudioTrack;
