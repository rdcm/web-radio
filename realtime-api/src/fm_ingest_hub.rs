use crate::app_state::AppState;
use crate::binary_codec::MsgpackCodec;
use crate::messages::AudioChunk;
use axum_signal::{MessageContext, WsHub};

pub struct FmIngestHub {
    state: AppState,
}

impl FmIngestHub {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl WsHub for FmIngestHub {
    type Codec = MsgpackCodec;
    type InMessage = AudioChunk;
    type OutMessage = AudioChunk;

    async fn on_message(&self, chunk: AudioChunk, _ctx: MessageContext<AudioChunk, MsgpackCodec>) {
        let _ = self.state.tx.send(chunk);
    }
}
