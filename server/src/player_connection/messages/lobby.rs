use uuid::Uuid;

use crate::{
    app_state::AppState,
    player_connection::messages::{error::send_error, structs::ServerMessage},
    structs::{Lobby, Player, PlayerStatus},
};

pub async fn create_new_lobby(player: Player, state: &AppState) {
    log::info!("Player {} is creating a new lobby", player.name);
    {
        if player.status != PlayerStatus::Idle {
            log::info!(
                "Player {} cannot create lobby while in status {:?}",
                player.name,
                player.status
            );
            send_error(
                player.id,
                "Cannot join lobby while in queue or in game".to_string(),
                state,
            )
            .await;
        }
    }
    let lobby_id = Uuid::new_v4();
    {
        let mut lobby = state.lobby.write().await;
        lobby.insert(lobby_id, Lobby::new(lobby_id, player.id));
    }
    join_existing_lobby(player, lobby_id, state).await;
}

pub async fn join_existing_lobby(player: Player, lobby_id: Uuid, state: &AppState) {
    log::info!("Player {} joining lobby {}", player.name, lobby_id);

    leave_lobby(player.clone(), state).await;

    if player.status != PlayerStatus::Idle {
        send_error(
            player.id,
            "Cannot join lobby while in queue or in game".to_string(),
            state,
        )
        .await;
        return;
    }

    let mut lobby = state.lobby.write().await;
    if let Some(l) = lobby.get_mut(&lobby_id) {
        let mut players = state.players.write().await;
        if let Some(p) = players.get_mut(&player.id) {
            p.lobby = Some(lobby_id);
        }
        l.players.push(player.id);
        state
            .send_to(player.id, ServerMessage::LobbyJoined { lobby_id })
            .await;
    } else {
        state
            .send_to(
                player.id,
                ServerMessage::Error {
                    message: "Lobby not found".to_string(),
                },
            )
            .await;
    }
}

pub async fn display_lobby(player_id: Uuid, state: &AppState) {
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

pub async fn leave_lobby(player: Player, state: &AppState) {
    let mut lobby = state.lobby.write().await;
    let players = state.players.read().await;
    if let Some(p) = players.get(&player.id) {
        if p.status != PlayerStatus::Idle {
            state
                .send_to(
                    player.id,
                    ServerMessage::Error {
                        message: "Cannot leave lobby while in queue or in game".to_string(),
                    },
                )
                .await;
            return;
        }
        let mut empty_lobby_id: Option<Uuid> = None;
        for l in lobby.values_mut() {
            if l.players.contains(&player.id) {
                l.players.retain(|&id| id != player.id);
                if l.players.is_empty() {
                    empty_lobby_id = Some(l.id);
                }

                if l.owner == player.id {
                    if let Some(&new_owner) = l.players.first() {
                        l.owner = new_owner;
                    } else {
                        empty_lobby_id = Some(l.id);
                    }
                }
                drop(players);
                let mut players = state.players.write().await;
                if let Some(p) = players.get_mut(&player.id) {
                    p.status = PlayerStatus::Idle;
                    p.lobby = None;
                }
                state
                    .send_to(player.id, ServerMessage::LobbyLeft { lobby_id: l.id })
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

async fn send_lobby(player_id: Uuid, lobby: &Lobby, state: &AppState) {
    let players;
    {
        let player_map = state.players.read().await;
        players = lobby
            .players
            .iter()
            .filter_map(|id| player_map.get(id))
            .map(|p| p.name.clone())
            .collect();
    }

    state
        .send_to(
            player_id,
            ServerMessage::Lobby {
                lobby_id: lobby.id,
                players,
                status: lobby.status.to_string(),
            },
        )
        .await;
}
