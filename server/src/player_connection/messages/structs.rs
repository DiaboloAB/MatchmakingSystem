use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::structs::Player;

#[derive(Debug, Serialize)]
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
    Researching {
        lobby_id: Uuid,
    },
    ResearchCancelled {
        lobby_id: Uuid,
        player_cancelling_id: String,
    },
    GameFound {
        match_id: Uuid,
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
    JoinLobby { id: Option<Uuid> },
    DisplayLobby,
    LeaveLobby,
    FindGame,
    CancelSearch,
    ConfirmGame { match_id: Uuid },
    Help,
}
