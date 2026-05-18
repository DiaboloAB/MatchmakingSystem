use std::collections::HashMap;

use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use uuid::Uuid;

use crate::{
    dashboard::messages::structs::{DashboardServerMessage, DashboardSnapshot},
    player_connection::messages::structs::ServerMessage,
    structs::{Game, Lobby, Player, PlayerStatus, QueueEntry},
};

// type PlayerTx = mpsc::UnboundedSender<ServerMessage>;

pub struct Settings {
    pub lobby_capacity: usize,
    pub team_size: usize,
    pub ranks: Vec<String>,
    pub div_number: usize,
    pub points_per_div: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lobby_capacity: 1,
            team_size: 1,
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
            points_per_div: 100,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub total_player: Arc<RwLock<usize>>,
    pub players: Arc<RwLock<HashMap<Uuid, Player>>>,
    pub senders: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<ServerMessage>>>>,
    pub db: SqlitePool,
    pub lobbys: Arc<RwLock<HashMap<Uuid, Lobby>>>,
    pub queueing_lobbys: Arc<RwLock<Vec<QueueEntry>>>,
    pub waiting_games: Arc<RwLock<HashMap<Uuid, Game>>>,
    pub ongoing_games: Arc<RwLock<HashMap<Uuid, Game>>>,

    pub dashboard_tx: Arc<RwLock<mpsc::UnboundedSender<DashboardServerMessage>>>,
    pub settings: Arc<RwLock<Settings>>,
    // pub queue: Arc<RwLock<Vec<Uuid>>>,
}

impl AppState {
    pub fn new(db: SqlitePool, total_player: usize) -> Self {
        Self {
            total_player: Arc::new(RwLock::new(total_player)),
            players: Arc::new(RwLock::new(HashMap::new())),
            senders: Arc::new(RwLock::new(HashMap::new())),
            lobbys: Arc::new(RwLock::new(HashMap::new())),
            queueing_lobbys: Arc::new(RwLock::new(Vec::new())),
            waiting_games: Arc::new(RwLock::new(HashMap::new())),
            ongoing_games: Arc::new(RwLock::new(HashMap::new())),
            db,
            dashboard_tx: Arc::new(RwLock::new(
                mpsc::unbounded_channel::<DashboardServerMessage>().0,
            )),
            settings: Arc::new(RwLock::new(Settings::default())),
        }
    }

    pub async fn send_to(&self, player_id: Uuid, msg: ServerMessage) {
        let senders = self.senders.read().await;
        if let Some(tx) = senders.get(&player_id) {
            let _ = tx.send(msg);
        }
    }

    pub async fn send_to_players(&self, player_ids: Vec<Uuid>, msg: ServerMessage) {
        for player_id in player_ids {
            self.send_to(player_id, msg.clone()).await;
        }
    }

    pub async fn queue_lobby(&self, entry: QueueEntry) {
        let mut queueing_lobbys = self.queueing_lobbys.write().await;
        queueing_lobbys.push(entry);
    }

    pub async fn dequeue_lobby(&self, lobby_id: Uuid) {
        let mut queueing_lobbys = self.queueing_lobbys.write().await;
        if let Some(pos) = queueing_lobbys
            .iter()
            .position(|entry| entry.lobby_id == lobby_id)
        {
            queueing_lobbys.remove(pos);
        }
    }

    pub async fn update_lobby_status(&self, lobby_id: Uuid, status: PlayerStatus) {
        let player_ids = {
            let mut lobbys = self.lobbys.write().await;
            let lobby = match lobbys.get_mut(&lobby_id) {
                Some(l) => l,
                None => return,
            };
            lobby.status = status.clone();
            lobby.players.clone()
        };

        self.update_players_status(player_ids, status).await;
    }

    pub async fn update_players_status(&self, player_ids: Vec<Uuid>, status: PlayerStatus) {
        let mut players = self.players.write().await;
        for player_id in player_ids {
            if let Some(p) = players.get_mut(&player_id) {
                p.status = status.clone();
            }
        }
    }
}
