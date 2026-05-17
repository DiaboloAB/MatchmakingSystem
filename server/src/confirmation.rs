use uuid::Uuid;

use crate::{app_state::AppState, player_connection::messages::structs::ServerMessage};

pub async fn confirmation_loop(state: AppState) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        check_confirmations(&state).await;
    }
}

async fn check_confirmations(state: &AppState) {
    let games = state.waiting_games.write().await;
    let mut to_remove: Vec<Uuid> = vec![];
    let settings = state.settings.read().await;

    for (game_id, game) in games.iter() {
        if game.confirmed.len() == settings.team_size * 2 {
            log::info!("Game {} confirmed by all players", game_id);
            state
                .send_to_players(
                    game.team1.clone(),
                    ServerMessage::GameStarting { game_id: *game_id },
                )
                .await;
            state
                .send_to_players(
                    game.team2.clone(),
                    ServerMessage::GameStarting { game_id: *game_id },
                )
                .await;

            state
                .ongoing_games
                .write()
                .await
                .insert(*game_id, game.clone());

            let mut ongoing_games = state.ongoing_games.write().await;
            ongoing_games.insert(*game_id, game.clone());

            to_remove.push(*game_id);
        }
    }

    let mut games = state.waiting_games.write().await;
    for game_id in to_remove {
        games.remove(&game_id);
    }
}
