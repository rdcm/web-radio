use crate::app_state::BandState;
use crate::binary_codec::MsgpackCodec;
use crate::messages::{SpectrumChunk, SpectrumSubscribe};
use axum_signal::{MessageContext, WsHub};

pub struct SpectrumListenHub {
    band: BandState,
}

impl SpectrumListenHub {
    pub fn new(band: BandState) -> Self {
        Self { band }
    }
}

impl WsHub for SpectrumListenHub {
    type Codec = MsgpackCodec;
    type InMessage = SpectrumSubscribe;
    type OutMessage = SpectrumChunk;

    async fn on_message(
        &self,
        _sub: SpectrumSubscribe,
        ctx: MessageContext<SpectrumChunk, MsgpackCodec>,
    ) {
        let mut rx = self.band.spectrum_tx.subscribe();
        while let Ok(chunk) = rx.recv().await {
            ctx.unicast(chunk).await;
        }
    }
}
