use uuid::Uuid;

use crate::{app_state::AppState, player_connection::messages::structs::ServerMessage};

pub async fn help(player_id: Uuid, state: &AppState) {
    state
        .send_to(
            player_id,
            ServerMessage::Info {
                message:  "Available commands: JoinLobby, DisplayLobby, LeaveLobby, FindGame, CancelSearch".to_string(),
            },
        )
        .await;
}
