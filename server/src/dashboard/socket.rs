use std::time::Duration;

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    dashboard::messages::structs::{DashboardServerMessage, DashboardSnapshot},
};

pub async fn dashboard_ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    log::info!("Dashboard connected");
    ws.on_upgrade(move |socket| handle_dashboard_connection(socket, state))
}

async fn handle_dashboard_connection(mut socket: WebSocket, state: AppState) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<DashboardServerMessage>();

    *state.dashboard_tx.write().await = tx;
    // let mut rx = state.dashboard_tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if ws_tx.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            if let Message::Text(text) = msg {
                log::info!("Received from dashboard: {}", text);
            }
        }
        log::info!("Dashboard disconnected");
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    log::info!("Dashboard disconnected");
}

pub async fn dashboard_broadcast_loop(state: AppState) {
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;

        let total_player = state.total_player.read().await;
        let players = state.players.read().await;
        let lobbys = state.lobbys.read().await;
        let games = state.ongoing_games.read().await;

        let snapshot = DashboardSnapshot {
            total_player: *total_player,
            connected_players: players.len(),
            lobby_number: lobbys.len(),
            game_number: games.len(),
        };

        let _ = state
            .dashboard_tx
            .write()
            .await
            .send(DashboardServerMessage::Snapshot { snapshot });
    }
}
