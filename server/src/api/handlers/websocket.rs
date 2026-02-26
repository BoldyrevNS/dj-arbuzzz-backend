use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{dto::response::websocket::WebSocketMessage, AppState};

pub fn websocket_router(app_state: Arc<AppState>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(websocket_handler))
        .routes(routes!(websocket_stream_dfpwm_handler))
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
    println!("[WebSocket] Connection attempt");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    println!("[WebSocket] Connection established");
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

#[utoipa::path(
    get,
    path = "/stream-dfpwm",
    tag = "WebSocket",
    responses(
        (status = 101, description = "WebSocket connection established for DFPWM audio streaming"),
    )
)]
async fn websocket_stream_dfpwm_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    println!("[WebSocket DFPWM] Connection attempt");
    ws.on_upgrade(|socket| handle_dfpwm_stream(socket, state))
}

async fn handle_dfpwm_stream(socket: WebSocket, state: Arc<AppState>) {
    println!("[WebSocket DFPWM] Connection established");
    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.services.radio_service.subscribe_dfpwm();

    // Send DFPWM audio chunks as binary messages
    let mut send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(chunk) => {
                    if sender.send(Message::Binary(chunk.into())).await.is_err() {
                        println!("[WebSocket DFPWM] Failed to send chunk, client disconnected");
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    println!(
                        "[WebSocket DFPWM] Client lagged, skipped {} chunks",
                        skipped
                    );
                    // Продолжаем работу, пропуская отставшие чанки
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    println!("[WebSocket DFPWM] Broadcast channel closed");
                    break;
                }
            }
        }
        println!("[WebSocket DFPWM] Send task terminated");
    });

    // Handle incoming messages (mainly for detecting close)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                println!("[WebSocket DFPWM] Client closed connection");
                break;
            }
        }
        println!("[WebSocket DFPWM] Receive task terminated");
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    println!("[WebSocket DFPWM] Connection closed");
}
