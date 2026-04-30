use crate::app_state::AppState;
use crate::control_hub::ControlHub;
use crate::fm_ingest_hub::FmIngestHub;
use crate::fm_listen_hub::FmListenHub;
use crate::messages::StationInfo;
use crate::spectrum_ingest_hub::SpectrumIngestHub;
use crate::spectrum_listen_hub::SpectrumListenHub;
use crate::tune_hub::TuneHub;
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use axum_signal::{WsHubConfig, serve_hub};
use tower_http::cors::CorsLayer;

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .merge(realtime_recognition_router())
        .route("/stations", get(get_stations).post(post_stations))
        .layer(CorsLayer::permissive())
        .with_state(state.clone())
}

async fn get_stations(State(state): State<AppState>) -> Json<Vec<StationInfo>> {
    Json(state.stations.read().await.clone())
}

async fn post_stations(
    State(state): State<AppState>,
    Json(list): Json<Vec<StationInfo>>,
) -> StatusCode {
    *state.stations.write().await = list;
    StatusCode::OK
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
        .route(
            "/ws/tune",
            get(
                |ws: WebSocketUpgrade, State(state): State<AppState>| async move {
                    let config = WsHubConfig::default();
                    ws.on_upgrade(move |socket| async move {
                        serve_hub(socket, TuneHub::new(state), &config).await
                    })
                },
            ),
        )
        .route(
            "/ws/control",
            get(
                |ws: WebSocketUpgrade, State(state): State<AppState>| async move {
                    let config = WsHubConfig::default();
                    ws.on_upgrade(move |socket| async move {
                        serve_hub(socket, ControlHub::new(state), &config).await
                    })
                },
            ),
        )
}
