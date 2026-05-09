#[cfg(not(target_arch = "wasm32"))]
use rodio::Source;
use sdf::{AUDIO_SPECTRUM_BINS, AudioAnalysis};
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::{AnalyserNode, AudioContext, HtmlAudioElement, MediaElementAudioSourceNode};

#[cfg(not(target_arch = "wasm32"))]
pub struct AudioTrack {
    base_path: PathBuf,
    output: rodio::MixerDeviceSink,
    player: Option<rodio::Player>,
    analysis_track: Option<DecodedTrack>,
    started_at: Option<Instant>,
    analysis: AudioAnalysis,
    volume: f32,
    current_track: Option<&'static str>,
}

#[cfg(not(target_arch = "wasm32"))]
struct DecodedTrack {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: usize,
    duration: f32,
}

#[cfg(target_arch = "wasm32")]
pub struct AudioTrack {
    base_path: String,
    context: AudioContext,
    analyser: AnalyserNode,
    frequency_data: Vec<u8>,
    analysis: AudioAnalysis,
    volume: f32,
    audio: Option<HtmlAudioElement>,
    source: Option<MediaElementAudioSourceNode>,
    current_track: Option<&'static str>,
}

#[cfg(not(target_arch = "wasm32"))]
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
            analysis: AudioAnalysis::default(),
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

        self.analysis = track.analyze(started_at.elapsed().as_secs_f32());
        self.analysis
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
        self.analysis = AudioAnalysis::default();
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
impl AudioTrack {
    pub fn new(base_path: impl Into<String>) -> Option<Self> {
        let context = match AudioContext::new() {
            Ok(context) => context,
            Err(error) => {
                log_js_error("Failed to create audio context", error);
                return None;
            }
        };
        let analyser = match context.create_analyser() {
            Ok(analyser) => analyser,
            Err(error) => {
                log_js_error("Failed to create audio analyser", error);
                return None;
            }
        };

        analyser.set_fft_size(2048);
        analyser.set_smoothing_time_constant(0.55);

        if let Err(error) = analyser.connect_with_audio_node(&context.destination()) {
            log_js_error("Failed to connect audio analyser", error);
            return None;
        }

        Some(Self {
            base_path: base_path.into(),
            context,
            analyser,
            frequency_data: vec![0; 1024],
            analysis: AudioAnalysis::default(),
            volume: 1.0,
            audio: None,
            source: None,
            current_track: None,
        })
    }

    pub fn play(&mut self, track: Option<&'static str>) {
        if self.current_track == track {
            if track.is_some() {
                self.resume_and_play();
            }
            return;
        }

        self.stop_current();
        self.current_track = track;

        let Some(track) = track else {
            return;
        };

        let path = self.track_path(track);
        let audio = match HtmlAudioElement::new_with_src(&path) {
            Ok(audio) => audio,
            Err(error) => {
                log_js_error("Failed to create audio element", error);
                self.current_track = None;
                return;
            }
        };

        audio.set_loop(true);
        audio.set_volume(self.volume as f64);

        let source = match self.context.create_media_element_source(&audio) {
            Ok(source) => source,
            Err(error) => {
                log_js_error("Failed to create media audio source", error);
                self.current_track = None;
                return;
            }
        };

        if let Err(error) = source.connect_with_audio_node(&self.analyser) {
            log_js_error("Failed to connect media audio source", error);
            self.current_track = None;
            return;
        }

        self.audio = Some(audio);
        self.source = Some(source);
        self.resume_and_play();
    }

    pub fn analysis(&mut self) -> AudioAnalysis {
        if self.current_track.is_none() {
            return AudioAnalysis::default();
        }

        let expected_len = self.analyser.frequency_bin_count() as usize;
        if self.frequency_data.len() != expected_len {
            self.frequency_data.resize(expected_len, 0);
        }

        self.analyser.get_byte_frequency_data(&mut self.frequency_data);
        self.analysis = frequency_bytes_to_analysis(&self.frequency_data, self.context.sample_rate());
        self.analysis
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);

        if let Some(audio) = &self.audio {
            audio.set_volume(self.volume as f64);
        }
    }

    fn stop_current(&mut self) {
        if let Some(audio) = &self.audio {
            let _ = audio.pause();
            audio.set_current_time(0.0);
        }

        if let Some(source) = &self.source {
            let _ = source.disconnect();
        }

        self.audio = None;
        self.source = None;
    }

    fn resume_and_play(&self) {
        let _ = self.context.resume();

        if let Some(audio) = &self.audio {
            let _ = audio.play();
        }
    }

    fn track_path(&self, track: &str) -> String {
        let base_path = self.base_path.trim_end_matches('/');

        if base_path.is_empty() {
            track.to_owned()
        } else {
            format!("{}/{}", base_path, track.trim_start_matches('/'))
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn log_js_error(message: &str, error: JsValue) {
    web_sys::console::error_2(&JsValue::from_str(message), &error);
}

#[cfg(not(target_arch = "wasm32"))]
fn log_frequency(bin: usize, bin_count: usize, sample_rate: u32) -> f32 {
    const MIN_FREQUENCY: f32 = 40.0;
    const MAX_FREQUENCY: f32 = 12_000.0;

    let nyquist = sample_rate as f32 * 0.5;
    let max_frequency = MAX_FREQUENCY.min(nyquist.max(MIN_FREQUENCY));
    let t = bin as f32 / (bin_count.saturating_sub(1).max(1)) as f32;

    MIN_FREQUENCY * (max_frequency / MIN_FREQUENCY).powf(t)
}

#[cfg(target_arch = "wasm32")]
fn frequency_bytes_to_analysis(data: &[u8], sample_rate: f32) -> AudioAnalysis {
    const MIN_FREQUENCY: f32 = 40.0;
    const MAX_FREQUENCY: f32 = 12_000.0;

    let mut spectrum = [0.0; AUDIO_SPECTRUM_BINS];

    if data.is_empty() {
        return AudioAnalysis::default();
    }

    let nyquist = sample_rate * 0.5;
    let max_frequency = MAX_FREQUENCY.min(nyquist.max(MIN_FREQUENCY));
    let frequency_to_index = |frequency: f32| -> usize {
        ((frequency / nyquist) * data.len() as f32)
            .floor()
            .clamp(0.0, (data.len() - 1) as f32) as usize
    };

    for (bin, value) in spectrum.iter_mut().enumerate() {
        let start_t = bin as f32 / AUDIO_SPECTRUM_BINS as f32;
        let end_t = (bin + 1) as f32 / AUDIO_SPECTRUM_BINS as f32;
        let start_frequency = MIN_FREQUENCY * (max_frequency / MIN_FREQUENCY).powf(start_t);
        let end_frequency = MIN_FREQUENCY * (max_frequency / MIN_FREQUENCY).powf(end_t);
        let start = frequency_to_index(start_frequency);
        let end = frequency_to_index(end_frequency).max(start + 1).min(data.len());
        let sum = data[start..end].iter().map(|value| *value as f32 / 255.0).sum::<f32>();

        *value = sum / (end - start) as f32;
    }

    AudioAnalysis::from_spectrum(spectrum)
}
