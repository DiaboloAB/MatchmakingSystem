use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::structs::{Game, Lobby, Player, QueueEntry};

// ─── Client → Server ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DashboardClientMessage {
    GetPlayers,
    GetLobbys,
    GetGames,
}

// ─── Server → Client ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum DashboardServerMessage {
    Snapshot {
        snapshot: DashboardSnapshot,
    },
    PlayerList {
        players: Vec<Player>,
    },
    LobbyList {
        lobbies: Vec<Lobby>,
        queueing_lobbies: Vec<QueueEntrySnapshot>,
    },
    GameList {
        waiting_games: Vec<GameSnapshot>,
        ongoing_games: Vec<GameSnapshot>,
    },
}

// ─── Live snapshot (broadcast loop) ──────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct DashboardSnapshot {
    pub total_player: usize,
    pub connected_players: usize,
    pub lobby_number: usize,
    pub game_number: usize,
    // pub queueing_lobbies: usize,
    // pub waiting_games: usize,
    // pub ongoing_games: usize,
}

// ─── Serializable wrappers for types that contain Instant ────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct QueueEntrySnapshot {
    pub lobby_id: Uuid,
    pub avg_mmr: f64,
    pub player_count: usize,
    pub queue_seconds: u64,
}

impl From<&QueueEntry> for QueueEntrySnapshot {
    fn from(e: &QueueEntry) -> Self {
        Self {
            lobby_id: e.lobby_id,
            avg_mmr: e.avg_mmr,
            player_count: e.player_count,
            queue_seconds: e.start_time.elapsed().as_secs(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct GameSnapshot {
    pub id: Uuid,
    pub team1: Vec<Uuid>,
    pub team2: Vec<Uuid>,
    pub lobbys: Vec<Uuid>,
    pub confirmed: Vec<Uuid>,
    pub elapsed_seconds: u64,
}

impl From<&Game> for GameSnapshot {
    fn from(g: &Game) -> Self {
        Self {
            id: g.id,
            team1: g.team1.clone(),
            team2: g.team2.clone(),
            lobbys: g.lobbys.clone(),
            confirmed: g.confirmed.clone(),
            elapsed_seconds: g.start_time.elapsed().as_secs(),
        }
    }
}
