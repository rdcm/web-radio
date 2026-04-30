use crate::app_state::BandState;
use crate::binary_codec::MsgpackCodec;
use crate::messages::SpectrumChunk;
use axum_signal::{MessageContext, WsHub};

pub struct SpectrumIngestHub {
    band: BandState,
}

impl SpectrumIngestHub {
    pub fn new(band: BandState) -> Self {
        Self { band }
    }
}

impl WsHub for SpectrumIngestHub {
    type Codec = MsgpackCodec;
    type InMessage = SpectrumChunk;
    type OutMessage = SpectrumChunk;

    async fn on_message(
        &self,
        chunk: SpectrumChunk,
        _ctx: MessageContext<SpectrumChunk, MsgpackCodec>,
    ) {
        let _ = self.band.spectrum_tx.send(chunk);
    }
}
