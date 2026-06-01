use uuid::Uuid;

use crate::{
    app_state::AppState,
    player_connection::messages::{
        game::{cancel_research, cancel_research_id},
        structs::ServerMessage,
    },
    structs::{Lobby, Player, PlayerStatus},
};

pub async fn create_new_lobby(player: Player, state: &AppState) {
    log::info!("Player {} is creating a new lobby", player.name);

    if player.status != PlayerStatus::Idle {
        log::info!(
            "Player {} cannot create lobby while in status {:?}",
            player.name,
            player.status
        );
        state
            .send_error(
                player.id,
                "Cannot join lobby while in queue or in game".to_string(),
            )
            .await;
    }

    let lobby_id = Uuid::new_v4();
    {
        let mut lobbys = state.lobbys.write().await;
        lobbys.insert(lobby_id, Lobby::new(lobby_id, player.id));
    }
    join_existing_lobby(player, lobby_id, state).await;
}

pub async fn join_existing_lobby(player: Player, lobby_id: Uuid, state: &AppState) {
    log::info!("Player {} joining lobby {}", player.name, lobby_id);

    if let Some(_) = player.lobby {
        leave_lobby(player.clone(), state).await;
    }

    if player.status != PlayerStatus::Idle {
        state
            .send_error(
                player.id,
                "Cannot join lobby while in queue or in game".to_string(),
            )
            .await;
        return;
    }

    let mut lobbys = state.lobbys.write().await;
    if let Some(l) = lobbys.get_mut(&lobby_id) {
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
            .send_error(player.id, "Lobby not found".to_string())
            .await;
    }
}

pub async fn display_lobby(player: Player, state: &AppState) {
    log::info!("Player {} is requesting lobby info", player.name);
    let lobbys = state.lobbys.read().await;
    for l in lobbys.values() {
        if l.players.contains(&player.id) {
            send_lobby(player.id, l, state).await;
            return;
        }
    }
    state
        .send_error(player.id, "You are not in a lobby".to_string())
        .await;
}

pub async fn leave_lobby(player: Player, state: &AppState) {
    log::info!("Player {} is leaving lobby", player.name);
    let mut lobbys = state.lobbys.write().await;

    if player.status != PlayerStatus::Idle {
        state
            .send_error(
                player.id,
                "Cannot leave lobby while in queue or in game".to_string(),
            )
            .await;
        return;
    }

    let lobby_id = match player.lobby {
        Some(id) => id,
        None => {
            state
                .send_error(player.id, "You are not in a lobby".to_string())
                .await;
            return;
        }
    };

    if let Some(l) = lobbys.get_mut(&lobby_id) {
        l.players.retain(|&id| id != player.id);

        if l.owner == player.id
            && let Some(&new_owner) = l.players.first()
        {
            l.owner = new_owner;
        }
        {
            let mut players = state.players.write().await;
            if let Some(p) = players.get_mut(&player.id) {
                p.status = PlayerStatus::Idle;
                p.lobby = None;
            }
        }
        if l.players.is_empty() {
            log::info!("Lobby {} is now empty, deleting", l.id);
            lobbys.remove(&lobby_id);
        }
    }

    state
        .send_to(player.id, ServerMessage::LobbyLeft { lobby_id })
        .await;
}

pub async fn remove_player_from_lobby(player: &Player, state: &AppState) {
    log::info!("Removing player {} from lobby", player.name);
    let mut lobbys = state.lobbys.write().await;

    let lobby_id = match player.lobby {
        Some(id) => id,
        None => {
            return;
        }
    };

    cancel_research_id(lobby_id, state).await;

    if let Some(l) = lobbys.get_mut(&lobby_id) {
        log::info!("Player {} was in lobby {}, removing", player.name, lobby_id);
        l.players.retain(|&id| id != player.id);

        if l.owner == player.id
            && let Some(&new_owner) = l.players.first()
        {
            l.owner = new_owner;
        }

        if l.players.is_empty() {
            log::info!("Lobby {} is now empty, deleting", l.id);
            lobbys.remove(&lobby_id);
        }
    }
}

async fn send_lobby(player_id: Uuid, lobby: &Lobby, state: &AppState) {
    log::info!("Sending lobby info to player {}", player_id);
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
