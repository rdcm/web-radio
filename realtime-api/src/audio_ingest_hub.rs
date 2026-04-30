use crate::app_state::BandState;
use crate::binary_codec::MsgpackCodec;
use crate::messages::AudioChunk;
use axum_signal::{MessageContext, WsHub};

pub struct AudioIngestHub {
    band: BandState,
}

impl AudioIngestHub {
    pub fn new(band: BandState) -> Self {
        Self { band }
    }
}

impl WsHub for AudioIngestHub {
    type Codec = MsgpackCodec;
    type InMessage = AudioChunk;
    type OutMessage = AudioChunk;

    async fn on_message(&self, chunk: AudioChunk, _ctx: MessageContext<AudioChunk, MsgpackCodec>) {
        let _ = self.band.audio_tx.send(chunk);
    }
}
