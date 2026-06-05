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
    player_connection::{
        db::{db_load_player, db_save_player},
        messages::{
            game::{cancel_research, confirm_game, find_game},
            lobby::{
                create_new_lobby, display_lobby, join_existing_lobby, leave_lobby,
                remove_player_from_lobby,
            },
            structs::{ClientMessage, ServerMessage},
        },
    },
    structs::Player,
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(player_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    // {
    //     log::info!("Checking player {} connecting", player_id);
    //     let players = state.players.read().await;
    //     if players.contains_key(&player_id) {
    //         log::warn!(
    //             "Player {} already connected, rejecting new connection",
    //             player_id
    //         );
    //         return Response::builder()
    //             .status(400)
    //             .body("Player ID already connected".into())
    //             .unwrap();
    //     }
    // }

    log::info!("Player {} connected", player_id);
    ws.on_upgrade(move |socket| handle_connection(socket, player_id, state))
}

pub async fn ws_handler_new_player(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    let player_id = Uuid::new_v4();

    {
        log::info!("Checking player {} connecting", player_id);
        let players = state.players.read().await;
        if players.contains_key(&player_id) {
            log::warn!(
                "Player {} already connected, rejecting new connection",
                player_id
            );
            return Response::builder()
                .status(400)
                .body("Player ID already connected".into())
                .unwrap();
        }
    }

    log::info!("Player {} connected", player_id);
    ws.on_upgrade(move |socket| handle_connection(socket, player_id, state))
}

pub async fn handle_connection(socket: WebSocket, player_id: Uuid, state: AppState) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    let tx_for_cleanup = tx.clone();

    let player = match db_load_player(&state.db, player_id).await {
        Some(p) => {
            log::info!("Loaded existing player: {}", p.name);
            p
        }
        None => {
            let p = Player::new(player_id);
            log::info!("Created new player: {}", p.name);
            db_save_player(&state.db, &p).await;

            {
                let mut total_player = state.total_player.write().await;
                *total_player += 1;
            }
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

    let is_active_connection = {
        let senders = state.senders.read().await;
        if let Some(current_tx) = senders.get(&player_id) {
            current_tx.same_channel(&tx_for_cleanup)
        } else {
            false
        }
    };

    if !is_active_connection {
        log::warn!("Connection dropped, but newer connection exists. Skipping cleanup.");
        return;
    }

    let final_player = {
        let players = state.players.read().await;
        match players.get(&player_id).cloned() {
            Some(p) => p,
            None => {
                log::warn!("Could not find player {} for cleanup", player_id);
                return;
            }
        }
    };

    log::info!("Player final state: {:?}", final_player);
    log::info!("Cleaning up player {} connection", player_id);

    cleanup(player_id, &state).await;

    remove_player_from_lobby(&final_player, &state).await;

    db_save_player(&state.db, &final_player).await;
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
            return;
        }
    };

    let player = {
        let players = state.players.read().await;
        match players.get(&player_id).cloned() {
            Some(p) => p,
            None => return,
        }
    };

    match msg {
        ClientMessage::PlayerInfo => player_info(player, state).await,
        ClientMessage::JoinLobby { id } => match id {
            Some(lobby_id) => join_existing_lobby(player, lobby_id, state).await,
            None => create_new_lobby(player, state).await,
        },
        ClientMessage::DisplayLobby => display_lobby(player, state).await,
        ClientMessage::LeaveLobby => leave_lobby(player, state).await,
        ClientMessage::SearchGame => find_game(player, state).await,
        ClientMessage::CancelSearch => cancel_research(player, state).await,
        ClientMessage::ConfirmGame { id } => confirm_game(player, id, state).await,
        _ => {
            log::warn!("Unhandled message from {}: {}", player_id, text);
        }
    }
}

async fn player_info(player: Player, state: &AppState) {
    log::info!("Player {} requested info", player.name);
    state
        .send_to(
            player.id,
            ServerMessage::PlayerInfo {
                player: player.into(),
            },
        )
        .await;
}
