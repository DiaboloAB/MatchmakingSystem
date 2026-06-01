use clap::Parser;
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(version, about = "MatchFlow Matchmaking Simulator")]
struct Args {
    #[arg(long, default_value_t = 12345)]
    port: u16,

    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    host: String,

    #[arg(short, long, default_value_t = 50)]
    count: usize,

    #[arg(long, default_value = "app_database.db")]
    db: String,

    #[arg(long, default_value_t = 0.8)]
    existing_ratio: f32,

    #[arg(long, default_value_t = false)]
    packet_loss: bool,
}

async fn load_existing_player_ids(db_path: &str, limit: usize) -> Vec<Uuid> {
    let url = format!("sqlite:{}?mode=ro", db_path); // read-only
    let db = match sqlx::SqlitePool::connect(&url).await {
        Ok(pool) => pool,
        Err(e) => {
            println!(
                "[Sim] Could not open DB ({}), all bots will be new players",
                e
            );
            return vec![];
        }
    };

    let rows = sqlx::query("SELECT id FROM players ORDER BY RANDOM() LIMIT ?")
        .bind(limit as i64)
        .fetch_all(&db)
        .await
        .unwrap_or_default();

    use sqlx::Row;
    let ids: Vec<Uuid> = rows
        .iter()
        .filter_map(|row| {
            let id_str: String = row.get("id");
            Uuid::parse_str(&id_str).ok()
        })
        .collect();

    println!("[Sim] Loaded {} existing players from DB", ids.len());
    db.close().await;
    ids
}

async fn simulate_bot(host: String, port: u16, bot_id: usize, player_id: Uuid, packet_loss: bool) {
    println!("Packet loss simulation: {}", packet_loss);
    loop {
        let url = format!("ws://{}:{}/ws/{}", host, port, player_id);

        let (ws_stream, _) = match connect_async(&url).await {
            Ok(stream) => stream,
            Err(e) => {
                println!(
                    "[Bot {}] Connection failed: {}. Retrying later...",
                    bot_id, e
                );
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        println!("[Bot {}] Connected with ID: {}", bot_id, player_id);
        let (mut ws_tx, mut ws_rx) = ws_stream.split();

        loop {
            let msg = match tokio::time::timeout(Duration::from_secs(200), ws_rx.next()).await {
                Ok(Some(Ok(Message::Text(text)))) => text,
                Ok(_) => break,
                Err(_) => {
                    println!("[Bot {}] Stuck for 200s, reconnecting", bot_id);
                    break;
                }
            };
            let msg = match serde_json::from_str::<serde_json::Value>(&msg) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let msg_type = match msg.get("type").and_then(|t| t.as_str()) {
                Some(t) => t,
                None => continue,
            };

            match msg_type {
                "Welcome" => {
                    sleep(Duration::from_millis(rand::random_range(500..2000))).await;

                    let req = serde_json::json!({ "type": "JoinLobby", "id": null });
                    if ws_tx
                        .send(Message::Text(req.to_string().into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                "LobbyJoined" => {
                    sleep(Duration::from_millis(rand::random_range(1000..4000))).await;

                    let req = serde_json::json!({ "type": "SearchGame" });
                    if ws_tx
                        .send(Message::Text(req.to_string().into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                "GameFound" => {
                    sleep(Duration::from_millis(rand::random_range(10..100))).await;

                    if let Some(game_id) = msg.get("game_id").and_then(|id| id.as_str()) {
                        let req = serde_json::json!({ "type": "ConfirmGame", "id": game_id });

                        if !packet_loss {
                            // send w/o constraints first to ensure it reaches
                            if ws_tx
                                .send(Message::Text(req.to_string().into()))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        } else {
                            println!(
                                "[Bot {}] Simulating packet loss/delay for ConfirmGame message",
                                bot_id
                            );
                            // send w/ simulated network constraints
                            if send_with_constraints(
                                &mut ws_tx,
                                Message::Text(req.to_string().into()),
                                35,
                                2000,
                            )
                            .await
                            .is_err()
                            {
                                break;
                            }
                        }
                    }
                }
                "GameResult" => {}
                "StatusUpdate" => {
                    let status = msg.get("status").and_then(|s| s.as_str()).unwrap_or("");
                    if status == "Idle" {
                        let choice = rand::random_range(0..100);

                        if choice < 10 {
                            println!("[Bot {}] Done playing for now. Disconnecting.", bot_id);
                            break;
                        } else if choice < 30 {
                            // 30 sec to 2 min break
                            let break_time = rand::random_range(30..120);
                            println!(
                                "[Bot {}] Taking a {}s break in lobby...",
                                bot_id, break_time
                            );
                            sleep(Duration::from_secs(break_time)).await;
                        } else {
                            sleep(Duration::from_millis(rand::random_range(1500..4000))).await;
                        }

                        let req = serde_json::json!({ "type": "SearchGame" });
                        if ws_tx
                            .send(Message::Text(req.to_string().into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        let offline_duration = rand::random_range(15..45);
        println!(
            "[Bot {}] Offline. Will log back in {} seconds.",
            bot_id, offline_duration
        );
        sleep(Duration::from_secs(offline_duration)).await;
    }
}

async fn send_with_constraints<S>(
    ws_tx: &mut S,
    msg: Message,
    loss_rate_percent: u32,
    max_latency_ms: u64,
) -> Result<(), ()>
where
    S: SinkExt<Message> + Unpin,
{
    if rand::random_range(0..100) < loss_rate_percent {
        println!("[Network Constraint] Packet dropped intentionally!");
        return Ok(());
    }

    if max_latency_ms > 0 {
        let delay = rand::random_range(10..max_latency_ms);
        sleep(Duration::from_millis(delay)).await;
    }

    if ws_tx.send(msg).await.is_err() {
        return Err(());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let existing_count = (args.count as f32 * args.existing_ratio) as usize;
    let new_count = args.count - existing_count;

    println!(
        "Starting {} bots ({} existing players, {} new) on {}:{}",
        args.count, existing_count, new_count, args.host, args.port
    );

    let existing_ids = load_existing_player_ids(&args.db, existing_count).await;
    let actual_existing = existing_ids.len();
    let actual_new = args.count - actual_existing;

    println!(
        "[Sim] Spawning {} existing + {} new players",
        actual_existing, actual_new
    );

    let mut handles = Vec::new();

    for (i, player_id) in existing_ids.into_iter().enumerate() {
        let host = args.host.clone();
        let port = args.port;
        let startup_delay = rand::random_range(10..2000);
        sleep(Duration::from_millis(startup_delay)).await;
        handles.push(tokio::spawn(async move {
            simulate_bot(host, port, i, player_id, args.packet_loss).await;
        }));
    }
    for i in actual_existing..actual_existing + actual_new {
        let host = args.host.clone();
        let port = args.port;
        let player_id = Uuid::new_v4();
        let startup_delay = rand::random_range(10..2000);
        sleep(Duration::from_millis(startup_delay)).await;
        handles.push(tokio::spawn(async move {
            simulate_bot(host, port, i, player_id, args.packet_loss).await;
        }));
    }

    futures::future::join_all(handles).await;
    Ok(())
}
