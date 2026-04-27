use crate::app_state::AppState;
use crate::binary_codec::MsgpackCodec;
use crate::messages::{SpectrumChunk, SpectrumSubscribe};
use axum_signal::{MessageContext, WsHub};

pub struct SpectrumListenHub {
    pub state: AppState,
}

impl SpectrumListenHub {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl WsHub for SpectrumListenHub {
    type Codec = MsgpackCodec;
    type InMessage = SpectrumSubscribe;
    type OutMessage = SpectrumChunk;

    async fn on_message(&self, _sub: SpectrumSubscribe, ctx: MessageContext<SpectrumChunk, MsgpackCodec>) {
        let mut rx = self.state.spectrum_tx.subscribe();
        loop {
            match rx.recv().await {
                Ok(chunk) => ctx.unicast(chunk).await,
                Err(_) => break,
            }
        }
    }
}
