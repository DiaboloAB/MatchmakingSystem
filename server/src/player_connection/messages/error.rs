use uuid::Uuid;

use crate::{app_state::AppState, player_connection::messages::structs::ServerMessage};

impl AppState {
    pub async fn send_error(&self, player_id: Uuid, message: String) {
        self.send_to(player_id, ServerMessage::Error { message })
            .await;
    }
}
