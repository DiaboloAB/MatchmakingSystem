use uuid::Uuid;

use crate::{
    app_state::{AppState, Settings},
    player_connection::messages::structs::ServerMessage,
    structs::{Game, PlayerStatus},
};

pub async fn matchmaking_loop(state: AppState) {
    log::info!("Matchmaker started");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        try_form_matches(&state).await;
    }
}

async fn try_form_matches(state: &AppState) {
    let mut queueing_lobbys = state.queueing_lobbys.write().await;

    if queueing_lobbys.len() < 2 {
        return;
    }

    for i in 0..queueing_lobbys.len() {
        for j in (i + 1)..queueing_lobbys.len() {
            let lobby1 = &queueing_lobbys[i];
            let lobby2 = &queueing_lobbys[j];

            if lobby1.avg_mmr - lobby2.avg_mmr < 100.0 {
                log::info!(
                    "Match found between lobby {} and lobby {}",
                    lobby1.lobby_id,
                    lobby2.lobby_id
                );
                let game_id = Uuid::new_v4();
                {
                    let lobbys = state.lobbys.write().await;
                    let l1 = lobbys.get(&lobby1.lobby_id).unwrap();
                    let l2 = lobbys.get(&lobby2.lobby_id).unwrap();

                    let l1players = l1.players.clone();
                    let l2players = l2.players.clone();
                    drop(lobbys);
                    let game = Game::new(
                        game_id,
                        l1players.clone(),
                        l2players.clone(),
                        vec![lobby1.lobby_id, lobby2.lobby_id],
                    );
                    state
                        .send_to_players(game.team1.clone(), ServerMessage::GameFound { game_id })
                        .await;
                    state
                        .send_to_players(game.team2.clone(), ServerMessage::GameFound { game_id })
                        .await;
                    let mut waiting_games = state.waiting_games.write().await;
                    waiting_games.insert(game.id, game.clone());
                }

                state
                    .update_lobby_status(
                        lobby1.lobby_id,
                        PlayerStatus::NeedConfirmation { game_id },
                    )
                    .await;
                state
                    .update_lobby_status(
                        lobby2.lobby_id,
                        PlayerStatus::NeedConfirmation { game_id },
                    )
                    .await;
                queueing_lobbys.remove(j);
                queueing_lobbys.remove(i);
                break;
            }
        }
    }
}
