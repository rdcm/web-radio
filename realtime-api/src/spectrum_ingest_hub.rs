use crate::app_state::AppState;
use crate::binary_codec::MsgpackCodec;
use crate::messages::SpectrumChunk;
use axum_signal::{MessageContext, WsHub};

pub struct SpectrumIngestHub {
    state: AppState,
}

impl SpectrumIngestHub {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl WsHub for SpectrumIngestHub {
    type Codec = MsgpackCodec;
    type InMessage = SpectrumChunk;
    type OutMessage = SpectrumChunk;

    async fn on_message(&self, chunk: SpectrumChunk, _ctx: MessageContext<SpectrumChunk, MsgpackCodec>) {
        let _ = self.state.spectrum_tx.send(chunk);
    }
}
