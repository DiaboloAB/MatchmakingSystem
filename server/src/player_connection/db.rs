use sqlx::SqlitePool;
use uuid::Uuid;

use crate::structs::{Player, PlayerStatus};

pub async fn db_load_player(db: &SqlitePool, id: Uuid) -> Option<Player> {
    let id_str = id.to_string();

    let row =
        sqlx::query("SELECT id, name, mmr, rank, div, points, skills FROM players WHERE id = ?")
            .bind(id_str)
            .fetch_optional(db)
            .await
            .ok()??;

    use sqlx::Row;

    Some(Player {
        id,
        name: row.get::<String, _>("name"),
        mmr: row.get::<f64, _>("mmr"),
        rank: row.get::<i64, _>("rank") as u8,
        div: row.get::<i64, _>("div") as u8,
        status: PlayerStatus::Idle,
        lobby: None,
        points: row.get::<i64, _>("points") as u32,
        skills: serde_json::from_str(&row.get::<String, _>("skills"))
            .unwrap_or_else(|_| Vec::new()),
    })
}

pub async fn db_save_player(db: &SqlitePool, player: &Player) {
    let id_str = player.id.to_string();

    let _ = sqlx::query(
        "INSERT INTO players (id, name, mmr, rank, div, points, skills) VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, mmr = excluded.mmr, rank = excluded.rank, div = excluded.div, points = excluded.points, skills = excluded.skills",
    )
    .bind(id_str)
    .bind(player.name.clone())
    .bind(player.mmr)
    .bind(player.rank as i64)
    .bind(player.div as i64)
    .bind(player.points as i64)
    .bind(serde_json::to_string(&player.skills).unwrap_or_else(|_| String::new()))
    .execute(db)
    .await;
}
