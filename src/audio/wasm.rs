use super::{AUDIO_SPECTRUM_BINS, AudioAnalysis};
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::js_sys::Uint8Array;
use web_sys::{AnalyserNode, AudioContext, HtmlAudioElement, MediaElementAudioSourceNode};

pub struct AudioTrack {
    base_path: String,
    context: AudioContext,
    analyser: AnalyserNode,
    frequency_data: Vec<u8>,
    frequency_data_js: Uint8Array,
    analysis: AudioAnalysis,
    volume: f32,
    audio: Option<HtmlAudioElement>,
    source: Option<MediaElementAudioSourceNode>,
    playing: Rc<Cell<bool>>,
    current_track: Option<&'static str>,
}

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
            frequency_data_js: Uint8Array::new_with_length(1024),
            analysis: AudioAnalysis::default(),
            volume: 1.0,
            audio: None,
            source: None,
            playing: Rc::new(Cell::new(false)),
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
        self.playing = Rc::new(Cell::new(false));

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
        if self.current_track.is_none() || !self.playing.get() {
            return AudioAnalysis::default();
        }

        let expected_len = self.analyser.frequency_bin_count() as usize;
        if self.frequency_data.len() != expected_len {
            self.frequency_data.resize(expected_len, 0);
            self.frequency_data_js = Uint8Array::new_with_length(expected_len as u32);
        }

        self.analyser
            .get_byte_frequency_data_with_u8_array(&self.frequency_data_js);
        self.frequency_data_js.copy_to(&mut self.frequency_data);
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
        self.playing.set(false);

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
        let context = self.context.clone();
        let Some(audio) = self.audio.clone() else {
            return;
        };
        let playing = Rc::clone(&self.playing);

        wasm_bindgen_futures::spawn_local(async move {
            match context.resume() {
                Ok(promise) => {
                    if let Err(error) = wasm_bindgen_futures::JsFuture::from(promise).await {
                        log_js_error("Failed to resume audio context", error);
                        return;
                    }
                }
                Err(error) => {
                    log_js_error("Failed to resume audio context", error);
                    return;
                }
            }

            match audio.play() {
                Ok(promise) => {
                    if let Err(error) = wasm_bindgen_futures::JsFuture::from(promise).await {
                        log_js_error("Failed to play audio", error);
                        playing.set(false);
                        return;
                    }

                    playing.set(true);
                }
                Err(error) => {
                    log_js_error("Failed to play audio", error);
                    playing.set(false);
                }
            }
        });
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

fn log_js_error(message: &str, error: JsValue) {
    web_sys::console::error_2(&JsValue::from_str(message), &error);
}

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
