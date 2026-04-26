use crate::app_state::AppState;
use crate::binary_codec::MsgpackCodec;
use crate::messages::{FmChunk, FmSubscribe};
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
    type InMessage = FmSubscribe;
    type OutMessage = FmChunk;

    async fn on_message(&self, sub: FmSubscribe, ctx: MessageContext<FmChunk, MsgpackCodec>) {
        let mut rx = self.state.fm_tx.subscribe();
        loop {
            match rx.recv().await {
                Ok(chunk) if chunk.freq == sub.freq => ctx.unicast(chunk).await,
                Ok(_) => {}
                Err(_) => break,
            }
        }
    }
}
