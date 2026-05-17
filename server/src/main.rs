use axum::{Router, routing::get};
use clap::Parser;
use sqlx::SqlitePool;
use tokio::signal;

use crate::{
    app_state::AppState,
    confirmation::confirmation_loop,
    dashboard::socket::{dashboard_broadcast_loop, dashboard_ws_handler},
    matchmaking::matchmaking_loop,
    player_connection::socket::{ws_handler, ws_handler_new},
};

mod app_state;
mod confirmation;
mod dashboard;
mod matchmaking;
mod player_connection;
mod structs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 12345)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    let db = SqlitePool::connect("sqlite:app_database.db?mode=rwc").await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS players (
            id   TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            mmr REAL NOT NULL DEFAULT 1000.0,
            rank INTEGER NOT NULL DEFAULT 0,
            div  INTEGER NOT NULL DEFAULT 1
        )",
    )
    .execute(&db)
    .await?;

    let total_player = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM players")
        .fetch_one(&db)
        .await
        .unwrap_or(0) as usize;

    let state = AppState::new(db, total_player);

    tokio::spawn(matchmaking_loop(state.clone()));
    tokio::spawn(confirmation_loop(state.clone()));
    tokio::spawn(dashboard_broadcast_loop(state.clone()));

    let app = Router::new()
        .route("/ws/{player_id}", get(ws_handler))
        .route("/ws_new/", get(ws_handler_new))
        .route("/ws/dashboard", get(dashboard_ws_handler)) // Add this line
        .with_state(state);

    let addr = format!("127.0.0.1:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    log::info!("Server listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(async { signal::ctrl_c().await.unwrap() })
        .await?;

    log::info!("Server shut down cleanly");
    Ok(())
}
