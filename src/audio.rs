#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::{AnalyserNode, AudioContext, HtmlAudioElement, MediaElementAudioSourceNode};

#[cfg(not(target_arch = "wasm32"))]
pub struct AudioTrack {
    base_path: PathBuf,
    output: rodio::MixerDeviceSink,
    player: Option<rodio::Player>,
    current_track: Option<&'static str>,
}

#[cfg(target_arch = "wasm32")]
pub struct AudioTrack {
    base_path: String,
    context: AudioContext,
    analyser: AnalyserNode,
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
            current_track: None,
        })
    }

    pub fn play(&mut self, track: Option<&'static str>) {
        if self.current_track == track {
            return;
        }

        self.player = None;
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
        self.player = Some(player);
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

        if let Err(error) = analyser.connect_with_audio_node(&context.destination()) {
            log_js_error("Failed to connect audio analyser", error);
            return None;
        }

        Some(Self {
            base_path: base_path.into(),
            context,
            analyser,
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
