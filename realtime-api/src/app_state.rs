use crate::messages::{AudioChunk, SpectrumChunk};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<AudioChunk>,
    pub spectrum_tx: broadcast::Sender<SpectrumChunk>,
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
        Self { tx, spectrum_tx }
    }
}
