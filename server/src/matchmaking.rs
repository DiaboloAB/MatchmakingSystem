use uuid::Uuid;

use crate::{
    app_state::{AppState, Settings},
    player_connection::messages::structs::ServerMessage,
    structs::{Game, PlayerStatus, QueueEntry},
};

pub async fn matchmaking_loop(state: AppState) {
    log::info!("Matchmaker started");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        try_form_matches(&state).await;
    }
}

fn lobby_queue_match(setting: &Settings, lobby1: &QueueEntry, lobby2: &QueueEntry) -> bool {
    let time_in_queue1 = lobby1.start_time.elapsed().as_secs_f32();
    let time_in_queue2 = lobby2.start_time.elapsed().as_secs_f32();
    let lowest_time = time_in_queue1.min(time_in_queue2);

    let mmr_diff = (lobby1.avg_mmr - lobby2.avg_mmr).abs();
    let dynamic_threshold =
        setting.matchmaking_delta + lowest_time * setting.matchmaking_time_factor;
    mmr_diff < dynamic_threshold as f64
}

async fn try_form_matches(state: &AppState) {
    let mut queueing_lobbys = state.queueing_lobbys.write().await;
    let settings = state.settings.read().await;

    if queueing_lobbys.len() < 2 {
        return;
    }

    for i in 0..queueing_lobbys.len() {
        for j in (i + 1)..queueing_lobbys.len() {
            let lobby1 = &queueing_lobbys[i];
            let lobby2 = &queueing_lobbys[j];

            if lobby_queue_match(&settings, lobby1, lobby2) {
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

                add_queue_time_sample(state, lobby1.start_time.elapsed().as_secs_f64()).await;

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

async fn add_queue_time_sample(state: &AppState, queue_time: f64) {
    let mut samples = state.debug_avg_queue_time.write().await;
    samples.push(queue_time);
    if samples.len() > 25 {
        samples.remove(0);
    }
}
