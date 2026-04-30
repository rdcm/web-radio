use crate::messages::{AudioChunk, SpectrumChunk, StationInfo, TuneCommand};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

#[derive(Clone)]
pub struct BandState {
    pub audio_tx:    broadcast::Sender<AudioChunk>,
    pub spectrum_tx: broadcast::Sender<SpectrumChunk>,
    pub stations:    Arc<RwLock<Vec<StationInfo>>>,
}

impl BandState {
    fn new() -> Self {
        let (audio_tx, _)    = broadcast::channel(256);
        let (spectrum_tx, _) = broadcast::channel(64);
        Self {
            audio_tx,
            spectrum_tx,
            stations: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub fm:      BandState,
    pub am:      BandState,
    pub tune_tx: broadcast::Sender<TuneCommand>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (tune_tx, _) = broadcast::channel(16);
        Self { fm: BandState::new(), am: BandState::new(), tune_tx }
    }

    pub fn band(&self, modulation: &str) -> Option<BandState> {
        match modulation {
            "fm" => Some(self.fm.clone()),
            "am" => Some(self.am.clone()),
            _ => None,
        }
    }
}
