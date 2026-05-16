use std::collections::HashMap;

use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use uuid::Uuid;

use crate::{
    dashboard::socket::DashboardSnapshot,
    player_connection::messages::structs::ServerMessage,
    structs::{Lobby, Player},
};

// type PlayerTx = mpsc::UnboundedSender<ServerMessage>;

pub struct Settings {
    lobby_capacity: usize,
    team_size: usize,
    ranks: Vec<String>,
    div_number: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lobby_capacity: 10,
            team_size: 5,
            ranks: vec![
                "Unranked".to_string(),
                "Iron".to_string(),
                "Bronze".to_string(),
                "Silver".to_string(),
                "Gold".to_string(),
                "Platinum".to_string(),
                "Emerald".to_string(),
                "Diamond".to_string(),
                "Master".to_string(),
                "Grandmaster".to_string(),
                "Challenger".to_string(),
            ],
            div_number: 4,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub total_player: Arc<RwLock<usize>>,
    pub players: Arc<RwLock<HashMap<Uuid, Player>>>,
    pub senders: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<ServerMessage>>>>,
    pub db: SqlitePool,
    pub lobby: Arc<RwLock<HashMap<Uuid, Lobby>>>,

    pub dashboard_tx: broadcast::Sender<DashboardSnapshot>,
    pub settings: Arc<RwLock<Settings>>,
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
            settings: Arc::new(RwLock::new(Settings::default())),
        }
    }

    pub async fn send_to(&self, player_id: Uuid, msg: ServerMessage) {
        let senders = self.senders.read().await;
        if let Some(tx) = senders.get(&player_id) {
            let _ = tx.send(msg);
        }
    }
}
