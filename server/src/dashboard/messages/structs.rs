use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::structs::{Game, Lobby, Player, PlayerStatus, QueueEntry};

// ─── Client → Server ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DashboardClientMessage {
    GetPlayers,
    GetLobbys,
    GetGames,
    UpdateSettings { settings: AppSettings },
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
        lobbys: Vec<Lobby>,
        queueing_lobby: Vec<QueueEntrySnapshot>,
    },
    GameList {
        waiting_games: Vec<GameSnapshot>,
        ongoing_games: Vec<GameSnapshot>,
    },
    SettingsUpdate {
        settings: AppSettings,
    },
}

// ─── Live snapshot (broadcast loop) ──────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct DashboardSnapshot {
    pub total_player: usize,
    pub connected_players: usize,

    // pub idle_players: usize,
    // pub queueing_players: usize,
    // pub need_confirmation_players: usize,
    // pub in_game_players: usize,
    pub lobby_number: usize,
    pub game_number: usize,
    pub queueing_lobbies: usize,
    pub waiting_games: usize,
    pub ongoing_games: usize,
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

impl From<Game> for GameSnapshot {
    fn from(game: Game) -> Self {
        GameSnapshot {
            id: game.id,
            team1: game.team1.clone(),
            team2: game.team2.clone(),
            lobbys: game.lobbys.clone(),
            confirmed: game.confirmed.clone(),
            elapsed_seconds: game.start_time.elapsed().as_secs(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub lobby_capacity: usize,
    pub team_size: usize,
    pub simulation_speed: f32,
    pub confirmation_time: f32,
}
