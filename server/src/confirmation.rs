use uuid::Uuid;

use crate::{
    app_state::AppState, player_connection::messages::structs::ServerMessage, structs::Game,
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
    drop(settings);

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

    log::info!("Moved {} games from waiting to ongoing", count);
}
