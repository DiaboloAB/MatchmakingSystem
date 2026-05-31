use std::fmt::Display;

use names::{Generator, Name};
use rand::{Rng, RngExt, rng};
use random_word::Lang;
use serde::Serialize;
use uuid::Uuid;

pub fn mmr_to_rank(mmr: f64) -> &'static str {
    match mmr as u32 {
        0..=799 => "Iron",
        800..=999 => "Bronze",
        1000..=1199 => "Silver",
        1200..=1399 => "Gold",
        1400..=1599 => "Platinum",
        1600..=1799 => "Diamond",
        _ => "Master",
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

#[derive(Debug, Clone)]
pub struct GameResult {
    pub id: Uuid,
    pub team1: Vec<Uuid>,
    pub team2: Vec<Uuid>,
    pub winner: u8,
    pub start_time: std::time::Instant,
    pub duration: std::time::Duration,
}
