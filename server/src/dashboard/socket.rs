use std::time::Duration;

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;

use crate::{
    app_state::AppState,
    dashboard::messages::structs::{
        DashboardClientMessage, DashboardServerMessage, DashboardSnapshot, GameSnapshot,
    },
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
        handle_get_players(&state).await;
        handle_get_lobbys(&state).await;
        handle_get_games(&state).await;
        while let Some(Ok(msg)) = ws_rx.next().await {
            if let Message::Text(text) = msg {
                log::info!("Received from dashboard: {}", text);
                handle_message(&text, &state_clone).await;
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
            queueing_lobbies: 0,
            waiting_games: 0,
            ongoing_games: 0,
        };

        let _ = state
            .dashboard_tx
            .write()
            .await
            .send(DashboardServerMessage::Snapshot { snapshot });
    }
}

async fn handle_message(text: &str, state: &AppState) {
    let msg = match serde_json::from_str::<DashboardClientMessage>(text) {
        Ok(m) => m,
        Err(_) => {
            log::warn!("Failed to parse message from {}: {}", "dashboard", text);
            return;
        }
    };

    match msg {
        DashboardClientMessage::GetGames => handle_get_games(state).await,
        DashboardClientMessage::GetLobbys => handle_get_lobbys(state).await,
        DashboardClientMessage::GetPlayers => handle_get_players(state).await,
        _ => log::warn!("Unknown message type from dashboard: {:?}", msg),
    }
}

async fn handle_get_players(state: &AppState) {
    let players = state.players.read().await;
    let _ = state
        .dashboard_tx
        .write()
        .await
        .send(DashboardServerMessage::PlayerList {
            players: players.values().cloned().collect(),
        });
}

async fn handle_get_lobbys(state: &AppState) {
    let lobbys = state.lobbys.read().await;
    let queueing_lobbys = state.queueing_lobbys.read().await;
    let _ = state
        .dashboard_tx
        .write()
        .await
        .send(DashboardServerMessage::LobbyList {
            lobbys: lobbys.values().cloned().collect(),
            queueing_lobby: queueing_lobbys.iter().map(|e| e.into()).collect(),
        });
}

async fn handle_get_games(state: &AppState) {
    let waiting_games = state.waiting_games.read().await;
    let ongoing_games = state.ongoing_games.read().await;
    let _ = state
        .dashboard_tx
        .write()
        .await
        .send(DashboardServerMessage::GameList {
            waiting_games: waiting_games
                .values()
                .map(|g| GameSnapshot::from(g.clone()))
                .collect(),
            ongoing_games: ongoing_games
                .values()
                .map(|g| GameSnapshot::from(g.clone()))
                .collect(),
        });
}
