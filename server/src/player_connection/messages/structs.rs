use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::structs::{Player, PlayerStatus};

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Welcome {
        player: Player,
    },
    LobbyJoined {
        lobby_id: Uuid,
    },
    Lobby {
        lobby_id: Uuid,
        players: Vec<String>,
        status: String,
    },
    LobbyLeft {
        lobby_id: Uuid,
    },
    SearchingGame {
        lobby_id: Uuid,
        player_searching: String,
    },
    SearchCancelled {
        lobby_id: Uuid,
        player_cancelling: String,
    },
    GameFound {
        game_id: Uuid,
    },
    GameStarting {
        game_id: Uuid,
    },
    GameCancelled {
        game_id: Uuid,
    },
    GameResult {
        game_id: Uuid,
        won: bool,
        mmr_change: f64,
        new_mmr: f64,
        new_rank: String,
    },
    PlayerUpdated {
        player: Player,
    },
    Error {
        message: String,
    },
    Info {
        message: String,
    },
    StatusUpdate {
        status: PlayerStatus,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    DisplayPlayer { id: Option<Uuid> },
    JoinLobby { id: Option<Uuid> },
    DisplayLobby,
    LeaveLobby,
    SearchGame,
    CancelSearch,
    ConfirmGame { id: Uuid },
}
