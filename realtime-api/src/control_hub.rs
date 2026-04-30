use crate::app_state::AppState;
use crate::binary_codec::MsgpackCodec;
use crate::messages::{ControlSubscribe, TuneCommand};
use axum_signal::{MessageContext, WsHub};

pub struct ControlHub {
    pub state: AppState,
}

impl ControlHub {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl WsHub for ControlHub {
    type Codec = MsgpackCodec;
    type InMessage = ControlSubscribe;
    type OutMessage = TuneCommand;

    async fn on_message(
        &self,
        _sub: ControlSubscribe,
        ctx: MessageContext<TuneCommand, MsgpackCodec>,
    ) {
        let mut rx = self.state.tune_tx.subscribe();
        while let Ok(cmd) = rx.recv().await {
            ctx.unicast(cmd).await;
        }
    }
}
