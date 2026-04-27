use crate::app_state::AppState;
use crate::binary_codec::MsgpackCodec;
use crate::messages::{AudioChunk, AudioSubscribe};
use axum_signal::{MessageContext, WsHub};

pub struct FmListenHub {
    pub state: AppState,
}

impl FmListenHub {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl WsHub for FmListenHub {
    type Codec = MsgpackCodec;
    type InMessage = AudioSubscribe;
    type OutMessage = AudioChunk;

    async fn on_message(&self, sub: AudioSubscribe, ctx: MessageContext<AudioChunk, MsgpackCodec>) {
        let mut rx = self.state.tx.subscribe();
        loop {
            match rx.recv().await {
                Ok(chunk) if chunk.freq == sub.freq => ctx.unicast(chunk).await,
                Ok(_) => {}
                Err(_) => break,
            }
        }
    }
}
