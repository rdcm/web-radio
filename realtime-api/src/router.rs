use crate::app_state::AppState;
use crate::fm_ingest_hub::FmIngestHub;
use crate::fm_listen_hub::FmListenHub;
use crate::spectrum_ingest_hub::SpectrumIngestHub;
use crate::spectrum_listen_hub::SpectrumListenHub;
use axum::Router;
use axum::extract::{State, WebSocketUpgrade};
use axum::routing::get;
use axum_signal::{WsHubConfig, serve_hub};

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .merge(realtime_recognition_router())
        .with_state(state.clone())
}

pub fn realtime_recognition_router() -> Router<AppState> {
    Router::new()
        .route(
            "/ws/ingest",
            get(
                |ws: WebSocketUpgrade, State(state): State<AppState>| async move {
                    let config = WsHubConfig::default();
                    ws.on_upgrade(move |socket| async move {
                        serve_hub(socket, FmIngestHub::new(state), &config).await
                    })
                },
            ),
        )
        .route(
            "/ws/listen",
            get(
                |ws: WebSocketUpgrade, State(state): State<AppState>| async move {
                    let config = WsHubConfig::default();
                    ws.on_upgrade(move |socket| async move {
                        serve_hub(socket, FmListenHub::new(state), &config).await
                    })
                },
            ),
        )
        .route(
            "/ws/ingest/spectrum",
            get(
                |ws: WebSocketUpgrade, State(state): State<AppState>| async move {
                    let config = WsHubConfig::default();
                    ws.on_upgrade(move |socket| async move {
                        serve_hub(socket, SpectrumIngestHub::new(state), &config).await
                    })
                },
            ),
        )
        .route(
            "/ws/spectrum",
            get(
                |ws: WebSocketUpgrade, State(state): State<AppState>| async move {
                    let config = WsHubConfig::default();
                    ws.on_upgrade(move |socket| async move {
                        serve_hub(socket, SpectrumListenHub::new(state), &config).await
                    })
                },
            ),
        )
}
