use std::sync::Arc;

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{AppState, dto::response::websocket::WebSocketMessage};

pub fn websocket_router(app_state: Arc<AppState>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(websocket_handler))
        .with_state(app_state)
}

#[utoipa::path(
    get,
    path = "/ws",
    tag = "WebSocket",
    responses(
        (status = 101, description = "WebSocket connection established"),
    )
)]
async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    tracing::info!("WebSocket connection attempt");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    tracing::info!("WebSocket connection established");
    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.services.radio_service.subscribe_events();

    // Send current state immediately
    if let Ok(current_track) = state.services.radio_service.get_current_track_ws().await {
        let msg = WebSocketMessage::CurrentTrack(current_track);
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = sender.send(Message::Text(json.into())).await;
        }
    }

    if let Ok(playlist) = state.services.playlist_service.get_playlist_ws().await {
        let msg = WebSocketMessage::Playlist(playlist);
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = sender.send(Message::Text(json.into())).await;
        }
    }

    // Handle messages
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
