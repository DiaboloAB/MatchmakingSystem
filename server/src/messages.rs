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
        players: Vec<Uuid>,
        status: String,
    },
    LobbyLeft {
        lobby_id: Uuid,
    },
    GameFound {
        match_id: Uuid,
    },
    Error {
        message: String,
    },
    Help {
        commands: Vec<(&'static str, &'static str)>,
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
