use rand::{Rng, RngExt, rng};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::structs::{GameResult, Player, PlayerStatus, mmr_to_rank};

pub async fn db_load_player(db: &SqlitePool, id: Uuid) -> Option<Player> {
    let id_str = id.to_string();
    log::info!("Loading player with ID: {}", id_str);

    let row = sqlx::query(
        "SELECT id, name, mmr, true_skill, wins, losses, debug_rank, debug_player_level, debug_player_form FROM players WHERE id = ?",
    )
    .bind(id_str)
    .fetch_optional(db)
    .await
    .ok()??;

    use sqlx::Row;

    Some(Player {
        id,
        name: row.get::<String, _>("name"),
        mmr: row.get::<f64, _>("mmr"),
        debug_rank: mmr_to_rank(row.get::<f64, _>("mmr")).to_string(),
        true_skill: row.get::<f64, _>("true_skill"),
        status: PlayerStatus::Idle,
        lobby: None,
        wins: serde_json::from_str(&row.get::<String, _>("wins")).unwrap_or_else(|_| Vec::new()),
        losses: serde_json::from_str(&row.get::<String, _>("losses"))
            .unwrap_or_else(|_| Vec::new()),

        debug_player_level: rng().random_range(0.0..=10.0),
        debug_player_form: rng().random_range(0.8..=1.2),
    })
}

pub async fn db_save_player(db: &SqlitePool, player: &Player) {
    let id_str = player.id.to_string();

    let result = sqlx::query(
        "INSERT INTO players (id, name, mmr, true_skill, wins, losses, debug_rank, debug_player_level, debug_player_form) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, mmr = excluded.mmr, true_skill = excluded.true_skill, wins = excluded.wins, losses = excluded.losses, debug_rank = excluded.debug_rank, debug_player_level = excluded.debug_player_level, debug_player_form = excluded.debug_player_form",
    )
    .bind(id_str)           
    .bind(player.name.clone())      
    .bind(player.mmr)       
    .bind(player.true_skill) 
    .bind(serde_json::to_string(&player.wins).unwrap_or_else(|_| String::new()))
    .bind(serde_json::to_string(&player.losses).unwrap_or_else(|_| String::new()))
    .bind(player.debug_rank.clone())    
    .bind(player.debug_player_level) 
    .bind(player.debug_player_form)
    .execute(db)
    .await;
    if let Err(e) = result {
        log::error!("Failed to save player {}: {}", player.id, e);
    }
}

pub async fn db_save_game_result(db: &SqlitePool, game_result: &GameResult) {
    let id_str = game_result.id.to_string();
    let team1_str = serde_json::to_string(&game_result.team1).unwrap_or_else(|_| String::new());
    let team2_str = serde_json::to_string(&game_result.team2).unwrap_or_else(|_| String::new());

    let _ = sqlx::query(
        "INSERT INTO games (id, team1, team2, winner, start_time, end_time) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id_str)
    .bind(team1_str)
    .bind(team2_str)
    .bind(game_result.winner as i64)
    .bind(game_result.start_time.elapsed().as_secs() as i64)
    .bind((game_result.start_time + game_result.duration).elapsed().as_secs() as i64)
    .execute(db)
    .await;
}
