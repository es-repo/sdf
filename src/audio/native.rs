use super::AudioAnalysis;
use super::analysis::{AudioSmoother, DecodedTrack};
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

        Self::new(samples, sample_rate, channels)
    }
}
