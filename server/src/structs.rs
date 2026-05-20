use std::fmt::Display;

use names::{Generator, Name};
use rand::{Rng, RngExt, rng};
use random_word::Lang;
use serde::Serialize;
use uuid::Uuid;

pub fn rank_from_u8(value: u8) -> String {
    match value {
        0 => "Iron".to_string(),
        1 => "Bronze".to_string(),
        2 => "Silver".to_string(),
        3 => "Gold".to_string(),
        4 => "Platinum".to_string(),
        5 => "Emerald".to_string(),
        6 => "Diamond".to_string(),
        7 => "Master".to_string(),
        8 => "Grandmaster".to_string(),
        9 => "Challenger".to_string(),
        _ => "Unknown".to_string(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub mmr: f64,
    pub rank: u8,
    pub div: u8,
    pub points: u32,
    pub status: PlayerStatus,
    pub lobby: Option<Uuid>,

    // simulation-only fields
    pub skills: Vec<String>,
}

impl Player {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            name: Generator::with_naming(Name::Plain).next().unwrap(),
            mmr: 1000.0,
            rank: 0,
            div: 4,
            points: 0,
            status: PlayerStatus::Idle,
            lobby: None,
            skills: create_skill_set(),
        }
    }
}

// For simulation purposes, we assign each player a random set of skills from a predefined pool
pub fn create_skill_set() -> Vec<String> {
    let nb_skills = rng().random_range(3..6);
    let mut skills = Vec::new();
    for _ in 0..nb_skills {
        skills.push(random_word::get(random_word::Lang::En).to_string());
    }
    skills
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
    pub team1: Vec<(Uuid, f32)>,
    pub team2: Vec<(Uuid, f32)>,
    pub winner: u8,
    pub start_time: std::time::Instant,
    pub duration: std::time::Duration,
}
