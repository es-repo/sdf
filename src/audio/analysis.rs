pub const AUDIO_SPECTRUM_BINS: usize = 64;

const WINDOW_SIZE: usize = 1024;
const MIN_FREQUENCY: f32 = 40.0;
const MAX_FREQUENCY: f32 = 12_000.0;
const SMOOTHING_FACTOR: f32 = 0.8;

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

pub(super) struct DecodedTrack {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: usize,
    duration: f32,
}

impl DecodedTrack {
    pub(super) fn new(samples: Vec<f32>, sample_rate: u32, channels: usize) -> Result<Self, String> {
        if samples.is_empty() || channels == 0 || sample_rate == 0 {
            return Err("decoded audio has no samples".to_owned());
        }

        let duration = samples.len() as f32 / channels as f32 / sample_rate as f32;

        Ok(Self {
            samples,
            sample_rate,
            channels,
            duration,
        })
    }

    pub(super) fn analyze(&self, time: f32) -> AudioAnalysis {
        let frame_count = self.samples.len() / self.channels;

        if frame_count == 0 || self.duration <= 0.0 {
            return AudioAnalysis::default();
        }

        let looped_time = time.rem_euclid(self.duration);
        let center_frame = (looped_time * self.sample_rate as f32) as usize % frame_count;
        let half_window = (WINDOW_SIZE / 2) % frame_count;
        let first_frame = (center_frame + frame_count - half_window) % frame_count;
        let mut spectrum = [0.0; AUDIO_SPECTRUM_BINS];

        for (bin, value) in spectrum.iter_mut().enumerate() {
            let frequency = log_frequency(bin, AUDIO_SPECTRUM_BINS, self.sample_rate);
            let k = ((frequency * WINDOW_SIZE as f32) / self.sample_rate as f32)
                .round()
                .clamp(1.0, (WINDOW_SIZE / 2 - 1) as f32);
            let coefficient = 2.0 * (2.0 * std::f32::consts::PI * k / WINDOW_SIZE as f32).cos();
            let mut q1 = 0.0;
            let mut q2 = 0.0;

            for offset in 0..WINDOW_SIZE {
                let frame = (first_frame + offset) % frame_count;
                let sample = self.mono_sample(frame);
                let q0 = coefficient * q1 - q2 + sample;
                q2 = q1;
                q1 = q0;
            }

            let power = (q1 * q1 + q2 * q2 - q1 * q2 * coefficient).max(0.0);
            *value = (power.sqrt() / WINDOW_SIZE as f32 * 8.0).clamp(0.0, 1.0);
        }

        AudioAnalysis::from_spectrum(spectrum)
    }

    fn mono_sample(&self, frame: usize) -> f32 {
        let start = frame * self.channels;
        let end = start + self.channels;
        self.samples[start..end].iter().sum::<f32>() / self.channels as f32
    }
}

pub(super) struct AudioSmoother {
    spectrum: [f32; AUDIO_SPECTRUM_BINS],
    initialized: bool,
}

impl AudioSmoother {
    pub(super) const fn new() -> Self {
        Self {
            spectrum: [0.0; AUDIO_SPECTRUM_BINS],
            initialized: false,
        }
    }

    pub(super) fn smooth(&mut self, analysis: AudioAnalysis) -> AudioAnalysis {
        if !self.initialized {
            self.spectrum = analysis.spectrum;
            self.initialized = true;

            return analysis;
        }

        for (previous, current) in self.spectrum.iter_mut().zip(analysis.spectrum) {
            *previous = *previous * SMOOTHING_FACTOR + current * (1.0 - SMOOTHING_FACTOR);
        }

        AudioAnalysis::from_spectrum(self.spectrum)
    }

    pub(super) fn reset(&mut self) {
        self.spectrum = [0.0; AUDIO_SPECTRUM_BINS];
        self.initialized = false;
    }
}

impl Default for AudioSmoother {
    fn default() -> Self {
        Self::new()
    }
}

fn log_frequency(bin: usize, bin_count: usize, sample_rate: u32) -> f32 {
    let nyquist = sample_rate as f32 * 0.5;
    let max_frequency = MAX_FREQUENCY.min(nyquist.max(MIN_FREQUENCY));
    let t = bin as f32 / (bin_count.saturating_sub(1).max(1)) as f32;

    MIN_FREQUENCY * (max_frequency / MIN_FREQUENCY).powf(t)
}

fn average_range(values: &[f32], start: usize, end: usize) -> f32 {
    let start = start.min(values.len());
    let end = end.min(values.len());

    if start >= end {
        return 0.0;
    }

    values[start..end].iter().sum::<f32>() / (end - start) as f32
}
