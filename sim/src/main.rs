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
}

async fn simulate_bot(host: String, port: u16, bot_id: usize) {
    let player_id = Uuid::new_v4();

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

        while let Some(Ok(Message::Text(text))) = ws_rx.next().await {
            let msg = match serde_json::from_str::<serde_json::Value>(&text) {
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
                    sleep(Duration::from_millis(rand::random_range(500..3000))).await;

                    if let Some(game_id) = msg.get("game_id").and_then(|id| id.as_str()) {
                        let req = serde_json::json!({ "type": "ConfirmGame", "id": game_id });
                        // if ws_tx
                        //     .send(Message::Text(req.to_string().into()))
                        //     .await
                        //     .is_err()
                        // {
                        //     break;
                        // }

                        // send with simulated network constraints
                        if send_with_constraints(
                            &mut ws_tx,
                            Message::Text(req.to_string().into()),
                            10,
                            2000,
                        )
                        .await
                        .is_err()
                        {
                            break;
                        }
                    }
                }
                "GameResult" => {
                    let choice = rand::random_range(0..100);

                    if choice < 10 {
                        println!("[Bot {}] Done playing for now. Disconnecting.", bot_id);
                        break;
                    } else if choice < 30 {
                        let break_time = rand::random_range(10..30);
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
    println!("Starting simulation with {} bots...", args.count);

    let mut handles = Vec::new();

    for i in 0..args.count {
        let host = args.host.clone();
        let port = args.port;

        let startup_delay = rand::random_range(10..2000);
        sleep(Duration::from_millis(startup_delay)).await;

        let handle = tokio::spawn(async move {
            simulate_bot(host, port, i).await;
        });

        handles.push(handle);
    }

    futures::future::join_all(handles).await;

    Ok(())
}
