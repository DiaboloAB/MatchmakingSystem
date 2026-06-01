use uuid::Uuid;

use crate::{
    app_state::AppState,
    confirmation,
    player_connection::messages::structs::ServerMessage,
    structs::{Game, PlayerStatus},
};

pub async fn confirmation_loop(state: AppState) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        check_confirmations(&state).await;
    }
}

async fn check_confirmations(state: &AppState) {
    let settings = state.settings.read().await;
    let required = settings.team_size * 2;
    let confirmation_time = settings.confirmation_time;
    drop(settings);

    if confirmation_time > 0.0 {
        // check for games that are not confirmed after confirmation_time
        let now = std::time::Instant::now();
        let mut to_remove = Vec::new();
        let confirmation_time = std::time::Duration::from_secs(confirmation_time as u64);
        {
            let waiting = state.waiting_games.read().await;
            for (game_id, game) in waiting.iter() {
                if now.duration_since(game.start_time) > confirmation_time {
                    to_remove.push(*game_id);
                    log::info!("Removing unconfirmed game: {}", game_id);
                    for lobby_id in &game.lobbys {
                        state
                            .update_lobby_status(*lobby_id, PlayerStatus::Idle)
                            .await;
                    }
                    add_successful_match_sample(&state, false).await;
                }
            }
        }

        {
            let mut waiting = state.waiting_games.write().await;
            for game_id in to_remove {
                waiting.remove(&game_id);
            }
        }
    }

    let confirmed_games: Vec<(Uuid, Game)> = {
        let games = state.waiting_games.read().await;
        games
            .iter()
            .filter(|(_, g)| g.confirmed.len() >= required)
            .map(|(id, g)| (*id, g.clone()))
            .collect()
    };

    if confirmed_games.is_empty() {
        return;
    }

    for (game_id, game) in &confirmed_games {
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

        add_successful_match_sample(&state, true).await;
    }

    let count = confirmed_games.len();

    {
        let mut waiting = state.waiting_games.write().await;
        let mut ongoing = state.ongoing_games.write().await;
        for (game_id, game) in confirmed_games {
            waiting.remove(&game_id);
            ongoing.insert(game_id, game);
        }
    }

    {
        for game in state.ongoing_games.read().await.values() {
            for lobby_id in &game.lobbys {
                state
                    .update_lobby_status(*lobby_id, PlayerStatus::InGame { game_id: game.id })
                    .await;
            }
        }
    }

    log::info!("Moved {} games from waiting to ongoing", count);
}

async fn add_successful_match_sample(state: &AppState, success: bool) {
    let mut samples = state.debug_successful_match_rate.write().await;
    samples.push(success);
    if samples.len() > 25 {
        samples.remove(0);
    }
}
