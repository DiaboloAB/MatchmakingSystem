use sqlx::SqlitePool;
use uuid::Uuid;

use crate::structs::{GameResult, Player, PlayerStatus};

pub async fn db_load_player(db: &SqlitePool, id: Uuid) -> Option<Player> {
    let id_str = id.to_string();

    let row = sqlx::query(
        "SELECT id, name, mmr, true_skill, wins, losses, skills FROM players WHERE id = ?",
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
        true_skill: row.get::<f64, _>("true_skill"),
        status: PlayerStatus::Idle,
        lobby: None,
        wins: serde_json::from_str(&row.get::<String, _>("wins")).unwrap_or_else(|_| Vec::new()),
        losses: serde_json::from_str(&row.get::<String, _>("losses"))
            .unwrap_or_else(|_| Vec::new()),

        skills: serde_json::from_str(&row.get::<String, _>("skills"))
            .unwrap_or_else(|_| Vec::new()),
    })
}

pub async fn db_save_player(db: &SqlitePool, player: &Player) {
    let id_str = player.id.to_string();

    let _ = sqlx::query(
        "INSERT INTO players (id, name, mmr, true_skill, wins, losses, skills) VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, mmr = excluded.mmr, true_skill = excluded.true_skill, wins = excluded.wins, losses = excluded.losses, skills = excluded.skills",
    )
    .bind(id_str)
    .bind(player.name.clone())
    .bind(player.mmr)
    
    .bind(serde_json::to_string(&player.wins).unwrap_or_else(|_| String::new()))
    .bind(serde_json::to_string(&player.losses).unwrap_or_else(|_| String::new()))
    .bind(serde_json::to_string(&player.skills).unwrap_or_else(|_| String::new()))
    .execute(db)
    .await;
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
