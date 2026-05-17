use std::time::Instant;

use uuid::Uuid;

use crate::{
    app_state::AppState,
    player_connection::messages::structs::ServerMessage,
    structs::{Player, PlayerStatus, QueueEntry},
};

pub async fn find_game(player: Player, state: &AppState) {
    log::info!("Player {} is searching for a game", player.name);
    let lobby_id = match player.lobby {
        Some(id) => id,
        None => {
            state
                .send_to(
                    player.id,
                    ServerMessage::Error {
                        message: "You are not in a lobby".into(),
                    },
                )
                .await;
            return;
        }
    };

    let avg_mmr = {
        let players = state.players.read().await;
        let lobbys = state.lobbys.read().await;
        let lobby = match lobbys.get(&lobby_id) {
            Some(l) => l,
            None => return,
        };
        lobby
            .players
            .iter()
            .filter_map(|id| players.get(id))
            .map(|p| p.mmr)
            .sum::<f64>()
            / lobby.players.len() as f64
    };

    let player_ids = {
        let mut lobbys = state.lobbys.write().await;
        let lobby = match lobbys.get_mut(&lobby_id) {
            Some(l) => l,
            None => return,
        };
        lobby.status = PlayerStatus::InQueue {
            queue_time: Instant::now(),
        };
        lobby.players.clone()
    };

    state
        .update_players_status(
            player_ids.clone(),
            PlayerStatus::InQueue {
                queue_time: Instant::now(),
            },
        )
        .await;
    state
        .queue_lobby(QueueEntry {
            lobby_id,
            avg_mmr,
            player_count: player_ids.len(),
            start_time: Instant::now(),
        })
        .await;

    state
        .send_to_players(
            player_ids,
            ServerMessage::SearchingGame {
                lobby_id,
                player_searching: player.name,
            },
        )
        .await;
}

pub async fn cancel_research(player: Player, state: &AppState) {
    log::info!("Player {} is canceling game search", player.name);
    let lobby_id = match player.lobby {
        Some(id) => id,
        None => {
            state
                .send_error(player.id, "You are not in a lobby".into())
                .await;
            return;
        }
    };

    state
        .update_lobby_status(lobby_id, PlayerStatus::Idle)
        .await;
    state.dequeue_lobby(lobby_id).await;
}

pub async fn confirm_game(player: Player, id: Uuid, state: &AppState) {
    log::info!("Player {} is confirming game {}", player.name, id);
    let _lobby_id = match player.lobby {
        Some(id) => id,
        None => {
            state
                .send_error(player.id, "You are not in a lobby".into())
                .await;
            return;
        }
    };

    let game_id = match player.status {
        PlayerStatus::NeedConfirmation { game_id } => game_id,
        _ => {
            state
                .send_error(player.id, "You are not in a game confirmation state".into())
                .await;
            return;
        }
    };
    if game_id != id {
        state
            .send_error(player.id, "You are not confirming the correct game".into())
            .await;
        return;
    }

    let mut waiting_games = state.waiting_games.write().await;
    let game = match waiting_games.get_mut(&game_id) {
        Some(g) => g,
        None => return,
    };

    if !game.confirmed.contains(&player.id) {
        game.confirmed.push(player.id);
    }
}
