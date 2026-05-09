use super::AudioAnalysis;
use super::analysis::{AudioSmoother, DecodedTrack};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::ArrayBuffer;
use web_sys::{AudioBuffer, AudioContext, HtmlAudioElement, Response};

pub struct AudioTrack {
    base_path: String,
    context: AudioContext,
    analysis_track: Rc<RefCell<Option<DecodedTrack>>>,
    smoother: AudioSmoother,
    volume: f32,
    audio: Option<HtmlAudioElement>,
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

        Some(Self {
            base_path: base_path.into(),
            context,
            analysis_track: Rc::new(RefCell::new(None)),
            smoother: AudioSmoother::new(),
            volume: 1.0,
            audio: None,
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
        self.analysis_track = Rc::new(RefCell::new(None));
        self.smoother.reset();

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

        self.audio = Some(audio);
        self.load_analysis_track(path);
        self.resume_and_play();
    }

    pub fn analysis(&mut self) -> AudioAnalysis {
        if self.current_track.is_none() || !self.playing.get() {
            return AudioAnalysis::default();
        }

        let Some(audio) = &self.audio else {
            return AudioAnalysis::default();
        };
        let analysis_track = self.analysis_track.borrow();
        let Some(track) = analysis_track.as_ref() else {
            return AudioAnalysis::default();
        };

        let analysis = track.analyze(audio.current_time() as f32);
        self.smoother.smooth(analysis)
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

        self.audio = None;
        self.analysis_track.borrow_mut().take();
        self.smoother.reset();
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
                    if let Err(error) = JsFuture::from(promise).await {
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
                    if let Err(error) = JsFuture::from(promise).await {
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

    fn load_analysis_track(&self, path: String) {
        let context = self.context.clone();
        let analysis_track = Rc::clone(&self.analysis_track);

        wasm_bindgen_futures::spawn_local(async move {
            match fetch_decoded_track(&context, &path).await {
                Ok(track) => {
                    analysis_track.replace(Some(track));
                }
                Err(error) => {
                    log_js_error("Failed to prepare audio analysis", error);
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

async fn fetch_decoded_track(context: &AudioContext, path: &str) -> Result<DecodedTrack, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("Window is not available"))?;
    let response = JsFuture::from(window.fetch_with_str(path)).await?;
    let response = response.dyn_into::<Response>()?;
    let array_buffer = JsFuture::from(response.array_buffer()?).await?;
    let array_buffer = array_buffer.dyn_into::<ArrayBuffer>()?;
    let audio_buffer = JsFuture::from(context.decode_audio_data(&array_buffer)?).await?;
    let audio_buffer = audio_buffer.dyn_into::<AudioBuffer>()?;

    audio_buffer_to_track(&audio_buffer).map_err(|error| JsValue::from_str(&error))
}

fn audio_buffer_to_track(buffer: &AudioBuffer) -> Result<DecodedTrack, String> {
    let channels = buffer.number_of_channels() as usize;
    let sample_rate = buffer.sample_rate() as u32;
    let frame_count = buffer.length() as usize;

    if channels == 0 || sample_rate == 0 || frame_count == 0 {
        return Err("decoded audio has no samples".to_owned());
    }

    let mut channel_data = Vec::with_capacity(channels);

    for channel in 0..channels {
        channel_data.push(
            buffer
                .get_channel_data(channel as u32)
                .map_err(|_| format!("failed to read audio channel {channel}"))?,
        );
    }

    let mut samples = Vec::with_capacity(frame_count * channels);

    for frame in 0..frame_count {
        for channel in &channel_data {
            samples.push(channel[frame]);
        }
    }

    DecodedTrack::new(samples, sample_rate, channels)
}

fn log_js_error(message: &str, error: JsValue) {
    web_sys::console::error_2(&JsValue::from_str(message), &error);
}
