use uuid::Uuid;

use crate::{
    app_state::AppState,
    player_connection::messages::structs::ServerMessage,
    structs::{Lobby, LobbyStatus, PlayerStatus},
};

pub async fn find_game(player_id: Uuid, state: &AppState) {
    let mut lobby = state.lobby.write().await;
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
                send_research(*pl, l, state).await;
            }
            return;
        }
    }
}

pub async fn cancel_research(player_id: Uuid, state: &AppState) {
    let mut lobby = state.lobby.write().await;
    for l in lobby.values_mut() {
        if l.players.contains(&player_id) {
            l.status = LobbyStatus::Idle;

            for pl in &l.players {
                let mut player = state.players.write().await;
                if let Some(p) = player.get_mut(pl) {
                    p.status = PlayerStatus::Idle;
                }
                state
                    .send_to(
                        *pl,
                        ServerMessage::ResearchCancelled {
                            lobby_id: l.id,
                            player_cancelling_id: player.get(&player_id).unwrap().name.clone(),
                        },
                    )
                    .await;
            }
            return;
        }
    }
}

async fn send_research(player_id: Uuid, lobby: &Lobby, state: &AppState) {
    state
        .send_to(player_id, ServerMessage::Researching { lobby_id: lobby.id })
        .await;
}
