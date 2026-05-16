use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use sqlx::SqlitePool;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    messages::{ClientMessage, ServerMessage},
    structs::{Lobby, LobbyStatus, Player, PlayerStatus},
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(player_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    log::info!("Player {} connected", player_id);
    ws.on_upgrade(move |socket| handle_connection(socket, player_id, state))
}

pub async fn ws_handler_new(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    let player_id = Uuid::new_v4();
    log::info!("Player {} connected", player_id);
    ws.on_upgrade(move |socket| handle_connection(socket, player_id, state))
}

async fn db_load_player(db: &SqlitePool, id: Uuid) -> Option<Player> {
    let id_str = id.to_string();

    let row = sqlx::query("SELECT id, name, mmr, rank, div FROM players WHERE id = ?")
        .bind(id_str)
        .fetch_optional(db)
        .await
        .ok()??;

    use sqlx::Row;

    Some(Player {
        id,
        name: row.get::<String, _>("name"),
        mmr: row.get::<f64, _>("mmr"),
        rank: row.get::<i64, _>("rank") as u8,
        div: row.get::<i64, _>("div") as u8,
        status: PlayerStatus::Idle,
        lobby: None,
    })
}

async fn db_save_player(db: &SqlitePool, player: &Player) {
    let id_str = player.id.to_string();

    let _ = sqlx::query(
        "INSERT INTO players (id, name, mmr, rank, div) VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, mmr = excluded.mmr, rank = excluded.rank, div = excluded.div",
    )
    .bind(id_str)
    .bind(player.name.clone())
    .bind(player.mmr)
    .bind(player.rank as i64)
    .bind(player.div as i64)
    .execute(db)
    .await;
}

pub async fn handle_connection(socket: WebSocket, player_id: Uuid, state: AppState) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    let player = match db_load_player(&state.db, player_id).await {
        Some(p) => p,
        None => {
            let p = Player::new(player_id);
            db_save_player(&state.db, &p).await;
            p
        }
    };

    {
        let mut players = state.players.write().await;
        players.insert(player_id, player.clone());
        let mut senders = state.senders.write().await;
        senders.insert(player_id, tx);
    }

    let welcome = serde_json::to_string(&ServerMessage::Welcome {
        player: player.clone(),
    })
    .unwrap();
    let _ = ws_tx.send(welcome.into()).await;

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
                handle_message(player_id, &text, &state_clone).await;
            }
        }
        log::info!("Player {} disconnected", player_id);
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    cleanup(player_id, &state).await;
}

async fn cleanup(player_id: Uuid, state: &AppState) {
    {
        let mut players = state.players.write().await;
        players.remove(&player_id);
        let mut senders = state.senders.write().await;
        senders.remove(&player_id);
    }
}

async fn handle_message(player_id: Uuid, text: &str, state: &AppState) {
    let msg = match serde_json::from_str::<ClientMessage>(text) {
        Ok(m) => m,
        Err(_) => {
            log::warn!("Failed to parse message from {}: {}", player_id, text);
            help(player_id, state).await;
            return;
        }
    };

    match msg {
        ClientMessage::JoinLobby { id } => match id {
            Some(lobby_id) => join_existing_lobby(player_id, lobby_id, &state).await,
            None => create_new_lobby(player_id, &state).await,
        },
        ClientMessage::DisplayLobby => {
            display_lobby(player_id, state).await;
        }
        ClientMessage::LeaveLobby => {
            leave_lobby(player_id, state).await;
        }
        ClientMessage::FindGame => {
            find_game(player_id, state).await;
        }
        _ => help(player_id, state).await,
    }
}

async fn help(player_id: Uuid, state: &AppState) {
    state
        .send_to(
            player_id,
            ServerMessage::Help {
                commands: vec![
                    ("JoinLobby", "Join the matchmaking lobby"),
                    ("LeaveLobby", "Leave the matchmaking lobby"),
                    ("StartResearch", "Start researching for a match"),
                    ("ConfirmMatch {match_id}", "Confirm a found match"),
                    ("DisplayQueue", "Show current matchmaking queue"),
                    ("Help", "Show this help message"),
                ],
            },
        )
        .await;
}

async fn create_new_lobby(player_id: Uuid, state: &AppState) {
    let mut players = state.players.write().await;
    if let Some(p) = players.get_mut(&player_id) {
        if p.status != PlayerStatus::Idle {
            state
                .send_to(
                    player_id,
                    ServerMessage::Error {
                        message: "Cannot leave lobby while in queue or in game".to_string(),
                    },
                )
                .await;
            return;
        }
    }
    let lobby_id = Uuid::new_v4();
    {
        let mut lobby = state.lobby.write().await;
        lobby.insert(lobby_id, Lobby::new(lobby_id, player_id));
    }
    join_existing_lobby(player_id, lobby_id, state).await;
}

async fn join_existing_lobby(player_id: Uuid, lobby_id: Uuid, state: &AppState) {
    let mut players = state.players.write().await;
    if let Some(p) = players.get_mut(&player_id) {
        if p.status != PlayerStatus::Idle {
            state
                .send_to(
                    player_id,
                    ServerMessage::Error {
                        message: "Cannot leave lobby while in queue or in game".to_string(),
                    },
                )
                .await;
            return;
        }
    }
    leave_lobby(player_id, state).await;
    let mut lobby = state.lobby.write().await;
    if let Some(l) = lobby.get_mut(&lobby_id) {
        let mut players = state.players.write().await;
        if let Some(p) = players.get_mut(&player_id) {
            p.lobby = Some(lobby_id);
        }
        l.players.push(player_id);
        state
            .send_to(player_id, ServerMessage::LobbyJoined { lobby_id })
            .await;
    } else {
        state
            .send_to(
                player_id,
                ServerMessage::Error {
                    message: "Lobby not found".to_string(),
                },
            )
            .await;
    }
}

async fn display_lobby(player_id: Uuid, state: &AppState) {
    let lobby = state.lobby.read().await;
    let player = state.players.read().await;
    if let Some(p) = player.get(&player_id) {
        for l in lobby.values() {
            if l.players.contains(&player_id) {
                send_lobby(player_id, l, state).await;
                return;
            }
        }
        state
            .send_to(
                player_id,
                ServerMessage::Error {
                    message: "You are not in a lobby".to_string(),
                },
            )
            .await;
    }
}

async fn leave_lobby(player_id: Uuid, state: &AppState) {
    let mut lobby = state.lobby.write().await;
    let player = state.players.read().await;
    if let Some(p) = player.get(&player_id) {
        if p.status != PlayerStatus::Idle {
            state
                .send_to(
                    player_id,
                    ServerMessage::Error {
                        message: "Cannot leave lobby while in queue or in game".to_string(),
                    },
                )
                .await;
            return;
        }
        let mut empty_lobby_id: Option<Uuid> = None;
        for l in lobby.values_mut() {
            if l.players.contains(&player_id) {
                l.players.retain(|&id| id != player_id);
                // if lobby is empty, remove it
                if l.players.is_empty() {
                    empty_lobby_id = Some(l.id);
                }

                // if player was owner, assign new owner or mark lobby for deletion
                if l.owner == player_id {
                    if let Some(&new_owner) = l.players.first() {
                        l.owner = new_owner;
                    } else {
                        empty_lobby_id = Some(l.id);
                    }
                }
                // change player status to idle
                drop(player);
                let mut player = state.players.write().await;
                if let Some(p) = player.get_mut(&player_id) {
                    p.status = PlayerStatus::Idle;
                    p.lobby = None;
                }
                state
                    .send_to(player_id, ServerMessage::LobbyLeft { lobby_id: l.id })
                    .await;

                if let Some(lobby_id) = empty_lobby_id {
                    lobby.remove(&lobby_id);
                }
                return;
            }
        }

        if empty_lobby_id.is_some() {
            lobby.remove(&empty_lobby_id.unwrap());
        }
    }
}

async fn find_game(player_id: Uuid, state: &AppState) {
    let mut lobby = state.lobby.write().await;
    let player = state.players.read().await;
    if let Some(p) = player.get(&player_id) {
        for l in lobby.values_mut() {
            if l.players.contains(&player_id) {
                l.status = LobbyStatus::InQueue {
                    queue_time: std::time::Instant::now(),
                };

                for pl in &l.players {
                    let mut player = state.players.write().await;
                    if let Some(p) = player.get_mut(pl) {
                        p.status = PlayerStatus::InQueue;
                    }
                    send_lobby(*pl, l, state).await;
                }
                return;
            }
        }
    }
}

async fn send_lobby(player_id: Uuid, lobby: &Lobby, state: &AppState) {
    state
        .send_to(
            player_id,
            ServerMessage::Lobby {
                lobby_id: lobby.id,
                players: lobby.players.clone(),
                status: lobby.status.to_string(),
            },
        )
        .await;
}
