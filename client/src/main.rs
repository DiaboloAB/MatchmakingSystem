use clap::Parser;
use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(version, about = "MatchFlow client")]
struct Args {
    #[arg(long, default_value_t = 12345)]
    port: u16,

    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    host: String,

    #[arg(long)]
    id: Option<Uuid>,
}

fn parse_command(line: &str) -> Result<serde_json::Value, String> {
    let mut parts = line.trim().splitn(2, ' ');
    let cmd = parts.next().unwrap_or("").trim();
    let arg = parts.next().map(str::trim);

    match cmd {
        "JoinLobby" => match arg {
            Some(id) => {
                let uuid = id
                    .parse::<Uuid>()
                    .map_err(|_| format!("Invalid UUID: {}", id))?;
                Ok(serde_json::json!({ "type": "JoinLobby", "id": uuid }))
            }
            None => Ok(serde_json::json!({ "type": "JoinLobby", "id": null })),
        },
        "DisplayLobby" => Ok(serde_json::json!({ "type": "DisplayLobby" })),
        "LeaveLobby" => Ok(serde_json::json!({ "type": "LeaveLobby" })),
        "SearchGame" => Ok(serde_json::json!({ "type": "SearchGame" })),
        "CancelSearch" => Ok(serde_json::json!({ "type": "CancelSearch" })),
        "ConfirmGame" => {
            let id = arg.ok_or_else(|| "Usage: ConfirmGame <uuid>".to_string())?;
            let uuid = id
                .parse::<Uuid>()
                .map_err(|_| format!("Invalid UUID: {}", id))?;
            Ok(serde_json::json!({ "type": "ConfirmGame", "id": uuid }))
        }
        "DisplayPlayer" => match arg {
            Some(id) => {
                let uuid = id
                    .parse::<Uuid>()
                    .map_err(|_| format!("Invalid UUID: {}", id))?;
                Ok(serde_json::json!({ "type": "DisplayPlayer", "id": uuid }))
            }
            None => Ok(serde_json::json!({ "type": "DisplayPlayer", "id": null })),
        },
        "Help" => Ok(serde_json::json!({ "type": "Help" })),
        "" => Err(String::new()),
        _ => Err(format!("Unknown command: {}", cmd)),
    }
}

fn print_help() {
    println!("Commands:");
    println!("  JoinLobby [uuid]      → join a new or existing lobby");
    println!("  DisplayLobby          → show current lobby");
    println!("  LeaveLobby            → leave current lobby");
    println!("  SearchGame            → start matchmaking");
    println!("  CancelSearch          → cancel matchmaking");
    println!("  ConfirmGame <uuid>    → confirm a found game");
    println!("  DisplayPlayer [uuid]  → show player info");
    println!("  Help                  → show this");
    println!("  help / h              → show this (local alias)");
    println!("  quit / q              → disconnect\n");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let player_id = args.id.unwrap_or_else(Uuid::new_v4);
    let url = format!("ws://{}:{}/ws/{}", args.host, args.port, player_id);

    println!("Connecting as {} ...", player_id);
    println!(
        "Tip: reuse this ID to keep your stats: --id {}\n",
        player_id
    );

    let (ws_stream, _) = connect_async(&url).await?;
    println!("Connected. Type 'h' for help.\n");

    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    // Receive task: print raw JSON from server
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = ws_rx.next().await {
            // Pretty-print if valid JSON, otherwise print raw
            match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(json) => println!("<< {}", serde_json::to_string_pretty(&json).unwrap()),
                Err(_) => println!("<< {}", text),
            }
        }
        println!("\n[disconnected from server]");
    });

    // Send task: read stdin, parse command, send JSON
    let send_task = tokio::spawn(async move {
        let mut lines = BufReader::new(tokio::io::stdin()).lines();

        loop {
            let line = match lines.next_line().await {
                Ok(Some(l)) => l,
                _ => break,
            };

            let trimmed = line.trim();

            match trimmed {
                "quit" | "q" => {
                    println!("Bye.");
                    break;
                }
                "help" | "h" => {
                    print_help();
                    continue;
                }
                _ => {}
            }

            match parse_command(trimmed) {
                Ok(json) => {
                    let text = serde_json::to_string(&json).unwrap();
                    println!(">> {}", text);
                    if ws_tx.send(Message::Text(text.into())).await.is_err() {
                        println!("[error] Send failed — server disconnected.");
                        break;
                    }
                }
                Err(e) if e.is_empty() => {}
                Err(e) => println!("[error] {}", e),
            }
        }
    });

    tokio::select! {
        _ = recv_task => {}
        _ = send_task => {}
    }

    Ok(())
}
