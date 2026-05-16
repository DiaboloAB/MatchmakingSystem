use clap::Parser;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(version, about = "MatchFlow client")]
struct Args {
    #[arg(long, default_value_t = 3000)]
    port: u16,

    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    host: String,

    /// Your player UUID — omit to generate a new one
    #[arg(long)]
    id: Option<Uuid>,
}

// ── Messages (mirrors server) ─────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(tag = "type")]
enum ClientMessage {
    JoinQueue,
    LeaveQueue,
    ConfirmMatch { match_id: Uuid },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum ServerMessage {
    Welcome {
        player: serde_json::Value,
    },
    QueueJoined,
    MatchFound {
        match_id: Uuid,
        opponent: String,
    },
    MatchStarting {
        match_id: Uuid,
    },
    MatchResult {
        match_id: Uuid,
        won: bool,
        mmr_change: f64,
        new_mmr: f64,
    },
    Error {
        message: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let player_id = args.id.unwrap_or_else(Uuid::new_v4);
    let url = format!("ws://{}:{}/ws/{}", args.host, args.port, player_id);

    println!("Connecting as {} ...", player_id);
    println!("Tip: reuse this ID to keep your MMR: --id {}\n", player_id);

    let (ws_stream, _) = connect_async(&url).await?;
    println!("Connected.\n");
    print_help();

    let (mut ws_tx, mut ws_rx) = ws_stream.split();
    let (match_tx, mut match_rx) = tokio::sync::mpsc::unbounded_channel::<Uuid>();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = ws_rx.next().await {
            match serde_json::from_str::<ServerMessage>(&text) {
                Ok(msg) => handle_server_msg(msg, &match_tx),
                Err(_) => println!("[raw] {}", text),
            }
        }
        println!("\n[disconnected from server]");
    });

    let send_task = tokio::spawn(async move {
        let mut lines = BufReader::new(tokio::io::stdin()).lines();
        let mut pending_match: Option<Uuid> = None;

        loop {
            // Pick up any match_id the receive task found
            while let Ok(id) = match_rx.try_recv() {
                pending_match = Some(id);
            }

            let line = match lines.next_line().await {
                Ok(Some(l)) => l.trim().to_string(),
                _ => break,
            };

            let msg = match line.as_str() {
                "q" | "queue" => ClientMessage::JoinQueue,
                "l" | "leave" => ClientMessage::LeaveQueue,
                "c" | "confirm" => match pending_match {
                    Some(id) => ClientMessage::ConfirmMatch { match_id: id },
                    None => {
                        println!("No match to confirm.");
                        continue;
                    }
                },
                "h" | "help" => {
                    print_help();
                    continue;
                }
                "x" | "quit" => {
                    println!("Bye.");
                    break;
                }
                _ => {
                    println!("Unknown — type 'h' for help.");
                    continue;
                }
            };

            let json = serde_json::to_string(&msg).unwrap();
            if ws_tx.send(Message::Text(json.into())).await.is_err() {
                println!("Send failed — server disconnected.");
                break;
            }
        }
    });

    tokio::select! {
        _ = recv_task => {}
        _ = send_task => {}
    }

    Ok(())
}

fn print_help() {
    println!("Commands:");
    println!("  q / queue    → join matchmaking queue");
    println!("  l / leave    → leave queue");
    println!("  c / confirm  → confirm a found match");
    println!("  h / help     → show this");
    println!("  x / quit     → disconnect\n");
}

fn handle_server_msg(msg: ServerMessage, match_tx: &tokio::sync::mpsc::UnboundedSender<Uuid>) {
    match msg {
        ServerMessage::Welcome { player } => {
            let mmr = player["mmr"].as_f64().unwrap_or(0.0);
            let rank = &player["rank"];
            println!("[server] Welcome back! MMR: {:.0}  Rank: {}", mmr, rank);
        }
        ServerMessage::QueueJoined => {
            println!("[server] In queue — waiting for a match...");
        }
        ServerMessage::MatchFound { match_id, opponent } => {
            println!("\n[server] Match found! vs {}", opponent);
            println!("         Type 'c' to confirm (15 seconds)");
            let _ = match_tx.send(match_id);
        }
        ServerMessage::MatchStarting { match_id } => {
            println!("[server] Match {} starting!", match_id);
        }
        ServerMessage::MatchResult {
            won,
            mmr_change,
            new_mmr,
            ..
        } => {
            let sign = if mmr_change >= 0.0 { "+" } else { "" };
            let outcome = if won { "WIN ✓" } else { "LOSS ✗" };
            println!(
                "\n[server] {} | {}{:.1} MMR → {:.0}",
                outcome, sign, mmr_change, new_mmr
            );
            println!("         Type 'q' to queue again.");
        }
        ServerMessage::Error { message } => {
            println!("[server] Error: {}", message);
        }
    }
}
