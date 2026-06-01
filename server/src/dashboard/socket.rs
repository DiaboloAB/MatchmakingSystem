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
        AppSettings, DashboardClientMessage, DashboardServerMessage, DashboardSnapshot,
        GameSnapshot,
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
        update_settings(&state).await;
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
        let total_game = state.total_game.read().await;
        let queueing_lobbys = state.queueing_lobbys.read().await;
        let waiting_games = state.waiting_games.read().await;
        let ongoing_games = state.ongoing_games.read().await;
        let average_queue_time = {
            let samples = state.debug_avg_queue_time.read().await;
            if samples.is_empty() {
                0.0
            } else {
                samples.iter().sum::<f64>() / samples.len() as f64
            }
        };
        let successful_match_rate = {
            let samples = state.debug_successful_match_rate.read().await;
            if samples.is_empty() {
                0.0
            } else {
                let successful = samples.iter().filter(|&&x| x).count();
                successful as f64 / samples.len() as f64
            }
        };

        let snapshot = DashboardSnapshot {
            total_player: *total_player,
            total_finished_game: *total_game,
            connected_players: players.len(),
            lobby_number: lobbys.len(),
            game_number: games.len(),
            queueing_lobbies: queueing_lobbys.len(),
            waiting_games: waiting_games.len(),
            ongoing_games: ongoing_games.len(),
            average_queue_time,
            successful_match_rate,
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
        DashboardClientMessage::UpdateSettings { settings } => {
            handle_update_settings(state, settings).await
        }
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

async fn update_settings(state: &AppState) {
    let app_settings = AppSettings {
        lobby_capacity: state.settings.read().await.lobby_capacity,
        team_size: state.settings.read().await.team_size,
        simulation_speed: state.settings.read().await.simulation_speed,
        confirmation_time: state.settings.read().await.confirmation_time,
        matchmaking_delta: state.settings.read().await.matchmaking_delta,
        matchmaking_time_factor: state.settings.read().await.matchmaking_time_factor,
    };
    let _ = state
        .dashboard_tx
        .write()
        .await
        .send(DashboardServerMessage::SettingsUpdate {
            settings: app_settings,
        });
}

async fn handle_update_settings(state: &AppState, new_settings: AppSettings) {
    {
        let mut settings = state.settings.write().await;
        settings.lobby_capacity = new_settings.lobby_capacity;
        settings.team_size = new_settings.team_size;
        settings.simulation_speed = new_settings.simulation_speed;
        settings.confirmation_time = new_settings.confirmation_time;
    }
    update_settings(state).await;
}
