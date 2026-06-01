use crate::{
    app_state::AppState,
    dashboard::messages::structs::DashboardServerMessage,
    player_connection::{
        db::{db_save_game_result, db_save_player},
        messages::structs::ServerMessage,
    },
    structs::{Game, GameResult, PlayerStatus, mmr_to_rank},
};
use rand::RngExt;
use uuid::Uuid;

pub async fn game_simulation_loop(state: AppState) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
        check_ongoing_games(&state).await;
    }
}

async fn check_ongoing_games(state: &AppState) {
    let sim_speed = {
        let settings = state.settings.read().await;
        settings.simulation_speed
    };
    let mut finished_games = Vec::new();
    {
        let mut ongoing = state.ongoing_games.write().await;

        ongoing.retain(|_, game| {
            // 20-40 minutes simulated time, scaled by sim_speed
            let mut time_limit = std::time::Duration::from_secs(rand::random_range(1200..2400));
            time_limit =
                std::time::Duration::from_secs_f64(time_limit.as_secs_f64() / sim_speed as f64);
            let elapsed_time = game.start_time.elapsed();
            if elapsed_time >= time_limit {
                log::info!(
                    "Game {} finished! (elapsed: {:.2?}, limit: {:.2?})",
                    game.id,
                    elapsed_time,
                    time_limit
                );
                finished_games.push(game.clone());
                false
            } else {
                true
            }
        });
    }

    for game in finished_games {
        resolve_game(state, game).await;
    }
}

async fn resolve_game(state: &AppState, game: Game) {
    let mut t1_score = 0.0;
    let mut t2_score = 0.0;

    {
        let players = state.players.read().await;

        for player_id in &game.team1 {
            if let Some(player) = players.get(player_id) {
                t1_score += player.debug_player_level + player.debug_player_form;
            }
        }

        for player_id in &game.team2 {
            if let Some(player) = players.get(player_id) {
                t2_score += player.debug_player_level + player.debug_player_form;
            }
        }
    }

    {
        let mut rng = rand::rng();
        t1_score += rng.random_range(0.0..20.0);
        t2_score += rng.random_range(0.0..20.0);
    }

    let winning_team = if t1_score > t2_score {
        "Team 1"
    } else {
        "Team 2"
    };

    log::info!(
        "Game {} resolved! Team {} wins ({} - {})",
        game.id,
        winning_team,
        t1_score,
        t2_score
    );

    for lobby_id in &game.lobbys {
        state
            .update_lobby_status(*lobby_id, PlayerStatus::Idle)
            .await;
    }

    let game_result: GameResult = GameResult {
        id: game.id,
        team1: game.team1.clone(),
        team2: game.team2.clone(),
        winner: if t1_score > t2_score { 1 } else { 2 },
        start_time: game.start_time,
        duration: game.start_time.elapsed(),
    };
    db_save_game_result(&state.db, &game_result).await;
    update_players(state, &game_result).await;
    send_new_game_to_dashboard(state, &game_result).await;
    {
        let mut total_game = state.total_game.write().await;
        *total_game += 1;
    }
}

const K_VISIBLE: f64 = 16.0 * 2.0;
const K_HIDDEN: f64 = 64.0 * 2.0;

async fn update_players(state: &AppState, game_result: &GameResult) {
    let (winners, losers) = match game_result.winner {
        1 => (&game_result.team1, &game_result.team2),
        _ => (&game_result.team2, &game_result.team1),
    };

    let mmr_delta = compute_mmr_delta(state, winners, losers).await;

    apply_results(state, game_result, winners, losers, mmr_delta).await;
}

async fn compute_mmr_delta(state: &AppState, winners: &[Uuid], losers: &[Uuid]) -> (f64, f64) {
    let players = state.players.read().await;

    let avg_skill = |team: &[Uuid]| -> f64 {
        let skills: Vec<f64> = team
            .iter()
            .filter_map(|id| players.get(id))
            .map(|p| p.true_skill)
            .collect();
        if skills.is_empty() {
            1000.0
        } else {
            skills.iter().sum::<f64>() / skills.len() as f64
        }
    };

    let winner_skill = avg_skill(winners);
    let loser_skill = avg_skill(losers);

    let expected = 1.0 / (1.0 + 10f64.powf((loser_skill - winner_skill) / 400.0));

    let visible_delta = K_VISIBLE * (1.0 - expected);
    let hidden_delta = K_HIDDEN * (1.0 - expected);

    (visible_delta, hidden_delta)
}

async fn apply_results(
    state: &AppState,
    game_result: &GameResult,
    winners: &[Uuid],
    losers: &[Uuid],
    (visible_delta, hidden_delta): (f64, f64),
) {
    let mut notifications = Vec::new();

    {
        let mut players = state.players.write().await;

        for (player_id, won) in winners
            .iter()
            .map(|id| (id, true))
            .chain(losers.iter().map(|id| (id, false)))
        {
            let Some(p) = players.get_mut(player_id) else {
                continue;
            };

            let sign = if won { 1.0 } else { -1.0 };
            p.mmr = (p.mmr + sign * visible_delta).max(0.0);
            p.true_skill = (p.true_skill + sign * hidden_delta).max(0.0);
            p.debug_rank = mmr_to_rank(p.mmr).to_string();
            p.status = PlayerStatus::Idle;

            if won {
                p.wins.push(game_result.id)
            } else {
                p.losses.push(game_result.id)
            }

            notifications.push((
                *player_id,
                ServerMessage::GameResult {
                    game_id: game_result.id,
                    won,
                    mmr_change: sign * visible_delta,
                    new_mmr: p.mmr,
                    new_rank: mmr_to_rank(p.mmr).to_string(),
                },
            ));
        }
    }

    let players = state.players.read().await;
    for (player_id, msg) in notifications {
        if let Some(p) = players.get(&player_id) {
            db_save_player(&state.db, p).await;
        }
        state.send_to(player_id, msg).await;
    }
}

async fn send_new_game_to_dashboard(state: &AppState, game: &GameResult) {
    let msg = DashboardServerMessage::NewGameResult { game: game.clone() };

    let dashboard_tx = state.dashboard_tx.read().await;
    let _ = dashboard_tx.send(msg);
}
