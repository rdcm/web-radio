use crate::app_state::AppState;
use crate::binary_codec::MsgpackCodec;
use crate::messages::FmChunk;
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
    type InMessage = FmChunk;
    type OutMessage = FmChunk;

    async fn on_message(&self, chunk: FmChunk, _ctx: MessageContext<FmChunk, MsgpackCodec>) {
        let _ = self.state.fm_tx.send(chunk);
    }
}
