use uuid::Uuid;

use crate::{app_state::AppState, player_connection::messages::structs::ServerMessage};

pub async fn send_error(player_id: Uuid, message: String, state: &AppState) {
    state
        .send_to(player_id, ServerMessage::Error { message })
        .await;
}
