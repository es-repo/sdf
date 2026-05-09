use super::analysis::{AudioSmoother, MAX_FREQUENCY, MIN_FREQUENCY};
use super::{AUDIO_SPECTRUM_BINS, AudioAnalysis};
use rodio::Source;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub struct AudioTrack {
    base_path: PathBuf,
    output: rodio::MixerDeviceSink,
    player: Option<rodio::Player>,
    analysis_track: Option<DecodedTrack>,
    started_at: Option<Instant>,
    smoother: AudioSmoother,
    volume: f32,
    current_track: Option<&'static str>,
}

struct DecodedTrack {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: usize,
    duration: f32,
}

impl AudioTrack {
    pub fn new(base_path: impl Into<PathBuf>) -> Option<Self> {
        let output = match rodio::DeviceSinkBuilder::open_default_sink() {
            Ok(output) => output,
            Err(error) => {
                eprintln!("Failed to open default audio output: {error}");
                return None;
            }
        };

        Some(Self {
            base_path: base_path.into(),
            output,
            player: None,
            analysis_track: None,
            started_at: None,
            smoother: AudioSmoother::new(),
            volume: 1.0,
            current_track: None,
        })
    }

    pub fn play(&mut self, track: Option<&'static str>) {
        if self.current_track == track {
            return;
        }

        self.stop_current();
        self.current_track = track;

        let Some(track) = track else {
            return;
        };

        let path = self.base_path.join(track);
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(error) => {
                eprintln!("Failed to open audio track `{}`: {error}", path.display());
                self.current_track = None;
                return;
            }
        };

        let source = match rodio::Decoder::new_looped(file) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("Failed to decode audio track `{}`: {error}", path.display());
                self.current_track = None;
                return;
            }
        };

        let player = rodio::Player::connect_new(self.output.mixer());
        player.append(source);
        player.set_volume(self.volume);
        self.player = Some(player);

        match DecodedTrack::load(&path) {
            Ok(analysis_track) => {
                self.analysis_track = Some(analysis_track);
                self.started_at = Some(Instant::now());
            }
            Err(error) => {
                eprintln!("Failed to prepare audio analysis `{}`: {error}", path.display());
            }
        }
    }

    pub fn analysis(&mut self) -> AudioAnalysis {
        let Some(track) = &self.analysis_track else {
            return AudioAnalysis::default();
        };
        let Some(started_at) = self.started_at else {
            return AudioAnalysis::default();
        };

        let analysis = track.analyze(started_at.elapsed().as_secs_f32());
        self.smoother.smooth(analysis)
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);

        if let Some(player) = &self.player {
            player.set_volume(self.volume);
        }
    }

    fn stop_current(&mut self) {
        self.player = None;
        self.analysis_track = None;
        self.started_at = None;
        self.smoother.reset();
    }
}

impl DecodedTrack {
    fn load(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|error| error.to_string())?;
        let decoder = rodio::Decoder::new(file).map_err(|error| error.to_string())?;
        let sample_rate = decoder.sample_rate().get();
        let channels = decoder.channels().get() as usize;
        let samples = decoder.collect::<Vec<_>>();

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

    fn analyze(&self, time: f32) -> AudioAnalysis {
        const WINDOW_SIZE: usize = 1024;

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

fn log_frequency(bin: usize, bin_count: usize, sample_rate: u32) -> f32 {
    let nyquist = sample_rate as f32 * 0.5;
    let max_frequency = MAX_FREQUENCY.min(nyquist.max(MIN_FREQUENCY));
    let t = bin as f32 / (bin_count.saturating_sub(1).max(1)) as f32;

    MIN_FREQUENCY * (max_frequency / MIN_FREQUENCY).powf(t)
}
