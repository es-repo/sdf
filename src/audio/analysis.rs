pub const AUDIO_SPECTRUM_BINS: usize = 64;
pub(super) const MIN_FREQUENCY: f32 = 40.0;
pub(super) const MAX_FREQUENCY: f32 = 12_000.0;

#[derive(Clone, Copy, Debug)]
pub struct AudioAnalysis {
    pub spectrum: [f32; AUDIO_SPECTRUM_BINS],
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub volume: f32,
}

impl AudioAnalysis {
    const BASS_END: usize = 20;
    const MID_END: usize = 46;

    pub const fn silence() -> Self {
        Self {
            spectrum: [0.0; AUDIO_SPECTRUM_BINS],
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            volume: 0.0,
        }
    }

    pub fn from_spectrum(mut spectrum: [f32; AUDIO_SPECTRUM_BINS]) -> Self {
        for value in &mut spectrum {
            *value = value.clamp(0.0, 1.0);
        }

        Self {
            bass: average_range(&spectrum, 0, Self::BASS_END),
            mid: average_range(&spectrum, Self::BASS_END, Self::MID_END),
            treble: average_range(&spectrum, Self::MID_END, AUDIO_SPECTRUM_BINS),
            volume: average_range(&spectrum, 0, AUDIO_SPECTRUM_BINS),
            spectrum,
        }
    }

    pub fn sample(&self, index: usize) -> f32 {
        self.spectrum[index.min(AUDIO_SPECTRUM_BINS - 1)]
    }
}

impl Default for AudioAnalysis {
    fn default() -> Self {
        Self::silence()
    }
}

fn average_range(values: &[f32], start: usize, end: usize) -> f32 {
    let start = start.min(values.len());
    let end = end.min(values.len());

    if start >= end {
        return 0.0;
    }

    values[start..end].iter().sum::<f32>() / (end - start) as f32
}
