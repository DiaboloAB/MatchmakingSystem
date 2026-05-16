use std::collections::HashMap;

use serde::Serialize;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use uuid::Uuid;

use crate::{
    dashboard::DashboardSnapshot,
    messages::ServerMessage,
    structs::{Lobby, Player},
};

// type PlayerTx = mpsc::UnboundedSender<ServerMessage>;

#[derive(Clone)]
pub struct AppState {
    pub total_player: Arc<RwLock<usize>>,
    pub players: Arc<RwLock<HashMap<Uuid, Player>>>,
    pub senders: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<ServerMessage>>>>,
    pub db: SqlitePool,
    pub lobby: Arc<RwLock<HashMap<Uuid, Lobby>>>,

    pub dashboard_tx: broadcast::Sender<DashboardSnapshot>,
    // pub queue: Arc<RwLock<Vec<Uuid>>>,
}

impl AppState {
    pub fn new(db: SqlitePool, total_player: usize) -> Self {
        let (dashboard_tx, _) = broadcast::channel(5);

        Self {
            total_player: Arc::new(RwLock::new(total_player)),
            players: Arc::new(RwLock::new(HashMap::new())),
            senders: Arc::new(RwLock::new(HashMap::new())),
            lobby: Arc::new(RwLock::new(HashMap::new())),
            db,
            dashboard_tx,
        }
    }

    pub async fn send_to(&self, player_id: Uuid, msg: ServerMessage) {
        let senders = self.senders.read().await;
        if let Some(tx) = senders.get(&player_id) {
            let _ = tx.send(msg);
        }
    }
}
