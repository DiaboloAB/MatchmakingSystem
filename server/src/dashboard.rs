use std::time::Duration;

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use sqlx::SqlitePool;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    messages::{ClientMessage, ServerMessage},
    structs::{Lobby, LobbyStatus, Player, PlayerStatus},
};

#[derive(Serialize, Clone, Default)]
pub struct DashboardSnapshot {
    total_player: usize,
    connected_players: usize,
    lobby_number: usize,
    game_number: usize,
}

pub async fn dashboard_ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    log::info!("Dashboard connected");
    ws.on_upgrade(move |socket| handle_dashboard_connection(socket, state))
}

async fn handle_dashboard_connection(mut socket: WebSocket, state: AppState) {
    let mut rx = state.dashboard_tx.subscribe();

    while let Ok(snapshot) = rx.recv().await {
        let json = serde_json::to_string(&snapshot).unwrap();
        if socket.send(Message::Text(json.into())).await.is_err() {
            break;
        }
    }

    log::info!("Dashboard disconnected");
}

pub async fn dashboard_broadcast_loop(state: AppState) {
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;

        let total_player = state.total_player.read().await;
        let players = state.players.read().await;
        let lobby = state.lobby.read().await;

        let snapshot = DashboardSnapshot {
            total_player: *total_player,
            connected_players: players.len(),
            lobby_number: lobby.len(),
            game_number: lobby
                .values()
                .filter(|l| l.status == LobbyStatus::InGame)
                .count(),
        };

        let _ = state.dashboard_tx.send(snapshot);
    }
}
