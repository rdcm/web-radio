use crate::messages::FmChunk;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub fm_tx: broadcast::Sender<FmChunk>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (fm_tx, _) = broadcast::channel(256);
        Self { fm_tx }
    }
}
