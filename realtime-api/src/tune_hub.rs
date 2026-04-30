use crate::app_state::AppState;
use crate::binary_codec::MsgpackCodec;
use crate::messages::TuneCommand;
use axum_signal::{MessageContext, WsHub};

pub struct TuneHub {
    state: AppState,
}

impl TuneHub {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl WsHub for TuneHub {
    type Codec = MsgpackCodec;
    type InMessage = TuneCommand;
    type OutMessage = TuneCommand;

    async fn on_message(&self, cmd: TuneCommand, _ctx: MessageContext<TuneCommand, MsgpackCodec>) {
        let _ = self.state.tune_tx.send(cmd);
    }
}
