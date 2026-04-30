use crate::messages::{AudioChunk, SpectrumChunk, StationInfo, TuneCommand};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<AudioChunk>,
    pub spectrum_tx: broadcast::Sender<SpectrumChunk>,
    pub tune_tx: broadcast::Sender<TuneCommand>,
    pub stations: Arc<RwLock<Vec<StationInfo>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        let (spectrum_tx, _) = broadcast::channel(64);
        let (tune_tx, _) = broadcast::channel(16);
        Self {
            tx,
            spectrum_tx,
            tune_tx,
            stations: Arc::new(RwLock::new(Vec::new())),
        }
    }
}
