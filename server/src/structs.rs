use std::fmt::Display;

use names::{Generator, Name};
use rand::{RngExt, rng};
use serde::Serialize;
use uuid::Uuid;

pub fn mmr_to_rank(mmr: f64) -> &'static str {
    match mmr as u32 {
        0..=999 => "Iron",
        1000..=1049 => "Bronze",
        1050..=1099 => "Silver",
        1100..=1149 => "Gold",
        1150..=1199 => "Platinum",
        1200..=1249 => "Emerald",
        1250..=1299 => "Diamond",
        1300..=1349 => "Master",
        1350..=1449 => "Grandmaster",
        1450..=u32::MAX => "Challenger",
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub mmr: f64,
    pub true_skill: f64,
    pub debug_rank: String,
    pub status: PlayerStatus,
    pub lobby: Option<Uuid>,
    pub wins: Vec<Uuid>,
    pub losses: Vec<Uuid>,

    pub debug_player_level: f32,
    pub debug_player_form: f32,
}

impl Player {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            name: Generator::with_naming(Name::Plain).next().unwrap(),
            mmr: 1000.0,
            true_skill: 1000.0,
            debug_rank: mmr_to_rank(1000.0).to_string(),
            status: PlayerStatus::Idle,
            lobby: None,
            wins: Vec::new(),
            losses: Vec::new(),

            debug_player_level: rng().random_range(0.0..=10.0),
            debug_player_form: rng().random_range(0.8..=1.2),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub name: String,
    pub mmr: f64,
    pub debug_rank: String,
    pub status: PlayerStatus,
    pub lobby: Option<Uuid>,
    pub wins: Vec<Uuid>,
    pub losses: Vec<Uuid>,
}

impl From<Player> for PlayerInfo {
    fn from(player: Player) -> Self {
        Self {
            id: player.id,
            name: player.name,
            mmr: player.mmr,
            debug_rank: player.debug_rank,
            status: player.status,
            lobby: player.lobby,
            wins: player.wins,
            losses: player.losses,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum PlayerStatus {
    Idle,
    InQueue {
        #[serde(skip_serializing)]
        queue_time: std::time::Instant,
    },
    NeedConfirmation {
        game_id: Uuid,
    },
    InGame {
        game_id: Uuid,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct Lobby {
    pub id: Uuid,
    pub players: Vec<Uuid>,
    pub owner: Uuid,
    pub status: PlayerStatus,
}

impl Lobby {
    pub fn new(id: Uuid, owner: Uuid) -> Self {
        Self {
            id,
            players: vec![],
            owner,
            status: PlayerStatus::Idle,
        }
    }
}

impl Display for PlayerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerStatus::Idle => write!(f, "Idle"),
            PlayerStatus::InQueue { .. } => write!(f, "In Queue"),
            PlayerStatus::NeedConfirmation { .. } => write!(f, "Need Confirmation"),
            PlayerStatus::InGame { .. } => write!(f, "In Game"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueueEntry {
    pub start_time: std::time::Instant,
    pub lobby_id: Uuid,
    pub avg_mmr: f64,
    pub player_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub id: Uuid,
    pub team1: Vec<Uuid>,
    pub team2: Vec<Uuid>,
    pub lobbys: Vec<Uuid>,
    pub confirmed: Vec<Uuid>,
    pub start_time: std::time::Instant,
    pub status: GameStatus,
}

impl Game {
    pub fn new(id: Uuid, team1: Vec<Uuid>, team2: Vec<Uuid>, lobbys: Vec<Uuid>) -> Self {
        Self {
            id,
            team1,
            team2,
            lobbys,
            start_time: std::time::Instant::now(),
            status: GameStatus::WaitingForConfirmation,
            confirmed: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameStatus {
    WaitingForConfirmation,
    Ongoing,
    Finished,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameResult {
    pub id: Uuid,
    pub team1: Vec<Uuid>,
    pub team2: Vec<Uuid>,
    pub winner: u8,
    #[serde(skip_serializing)]
    pub start_time: std::time::Instant,
    #[serde(skip_serializing)]
    pub duration: std::time::Duration,
}
