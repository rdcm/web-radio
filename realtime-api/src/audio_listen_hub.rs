use crate::app_state::BandState;
use crate::binary_codec::MsgpackCodec;
use crate::messages::{AudioChunk, AudioSubscribe};
use axum_signal::{MessageContext, WsHub};

pub struct AudioListenHub {
    band: BandState,
}

impl AudioListenHub {
    pub fn new(band: BandState) -> Self {
        Self { band }
    }
}

impl WsHub for AudioListenHub {
    type Codec = MsgpackCodec;
    type InMessage = AudioSubscribe;
    type OutMessage = AudioChunk;

    async fn on_message(&self, sub: AudioSubscribe, ctx: MessageContext<AudioChunk, MsgpackCodec>) {
        let mut rx = self.band.audio_tx.subscribe();
        loop {
            match rx.recv().await {
                Ok(chunk) if chunk.freq == sub.freq => ctx.unicast(chunk).await,
                Ok(_) => {}
                Err(_) => break,
            }
        }
    }
}
