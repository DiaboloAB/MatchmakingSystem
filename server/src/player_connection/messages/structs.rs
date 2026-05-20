use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::structs::Player;

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
    Error {
        message: String,
    },
    Info {
        message: String,
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
