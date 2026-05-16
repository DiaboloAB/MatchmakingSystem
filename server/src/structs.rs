use std::fmt::Display;

use names::{Generator, Name};
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
    pub status: PlayerStatus,
    pub lobby: Option<Uuid>,
}

impl Player {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            name: Generator::with_naming(Name::Plain).next().unwrap(),
            mmr: 1000.0,
            rank: 0,
            div: 4,
            status: PlayerStatus::Idle,
            lobby: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum PlayerStatus {
    Idle,
    InQueue,
    InGame { match_id: Uuid },
    Finished,
}

#[derive(Debug, Clone, Serialize)]
pub struct Lobby {
    pub id: Uuid,
    pub players: Vec<Uuid>,
    pub owner: Uuid,
    pub status: LobbyStatus,
}

impl Lobby {
    pub fn new(id: Uuid, owner: Uuid) -> Self {
        Self {
            id,
            players: vec![],
            owner,
            status: LobbyStatus::Idle,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum LobbyStatus {
    Idle,
    InQueue {
        #[serde(skip_serializing)]
        queue_time: std::time::Instant,
    },
    FoundMatch {
        match_id: Uuid,
    },
    InGame,
}

impl Display for LobbyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LobbyStatus::Idle => write!(f, "Idle"),
            LobbyStatus::InQueue { .. } => write!(f, "In Queue"),
            LobbyStatus::FoundMatch { .. } => write!(f, "Match Found"),
            LobbyStatus::InGame => write!(f, "In Game"),
        }
    }
}
