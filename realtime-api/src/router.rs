use crate::app_state::AppState;
use crate::audio_ingest_hub::AudioIngestHub;
use crate::audio_listen_hub::AudioListenHub;
use crate::control_hub::ControlHub;
use crate::messages::StationInfo;
use crate::spectrum_ingest_hub::SpectrumIngestHub;
use crate::spectrum_listen_hub::SpectrumListenHub;
use crate::tune_hub::TuneHub;
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use axum_signal::{WsHubConfig, serve_hub};
use tower_http::cors::CorsLayer;

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/stations/{modulation}", get(get_stations).post(post_stations))
        .route("/ws/ingest/{modulation}", get(audio_ingest_ws))
        .route("/ws/ingest/{modulation}/spectrum", get(spectrum_ingest_ws))
        .route("/ws/listen/{modulation}", get(audio_listen_ws))
        .route("/ws/spectrum/{modulation}", get(spectrum_listen_ws))
        .route("/ws/tune", get(tune_ws))
        .route("/ws/control", get(control_ws))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn get_stations(
    Path(modulation): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.band(&modulation) {
        Some(band) => Json(band.stations.read().await.clone()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn post_stations(
    Path(modulation): Path<String>,
    State(state): State<AppState>,
    Json(list): Json<Vec<StationInfo>>,
) -> StatusCode {
    match state.band(&modulation) {
        Some(band) => {
            *band.stations.write().await = list;
            StatusCode::OK
        }
        None => StatusCode::NOT_FOUND,
    }
}

async fn audio_ingest_ws(
    Path(modulation): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    match state.band(&modulation) {
        Some(band) => ws
            .on_upgrade(move |socket| async move {
                serve_hub(socket, AudioIngestHub::new(band), &WsHubConfig::default()).await
            })
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn spectrum_ingest_ws(
    Path(modulation): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    match state.band(&modulation) {
        Some(band) => ws
            .on_upgrade(move |socket| async move {
                serve_hub(socket, SpectrumIngestHub::new(band), &WsHubConfig::default()).await
            })
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn audio_listen_ws(
    Path(modulation): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    match state.band(&modulation) {
        Some(band) => ws
            .on_upgrade(move |socket| async move {
                serve_hub(socket, AudioListenHub::new(band), &WsHubConfig::default()).await
            })
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn spectrum_listen_ws(
    Path(modulation): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    match state.band(&modulation) {
        Some(band) => ws
            .on_upgrade(move |socket| async move {
                serve_hub(socket, SpectrumListenHub::new(band), &WsHubConfig::default()).await
            })
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn tune_ws(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| async move {
        serve_hub(socket, TuneHub::new(state), &WsHubConfig::default()).await
    })
    .into_response()
}

async fn control_ws(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| async move {
        serve_hub(socket, ControlHub::new(state), &WsHubConfig::default()).await
    })
    .into_response()
}
