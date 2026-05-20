# Matchmaking Server & Simulation

Here is a matchmaking server built with Rust, Axum, and Tokio. 
This project features a central game server, a dashboard for real-time monitoring, and a client simulator to test matchmaking logic and network resilience.

The goal is to design a system, here are the different parts of the project:
- a data source: player connections, matchmaking requests, and game results are stored in an SQLite database using `sqlx`.
- a data transmission layer: WebSockets are used for real-time communication between clients, the server, and the dashboard.
- a data collection: the server collects metrics on matchmaking times, queue lengths, and game outcomes, which are streamed to the dashboard for visualization.
- an ai-based processing layer: I thought about implementing an based algorithm, but it's not in the scope of this project, so I implemented a simple Elo-based matchmaking system instead.
- a final decision layer: the server runs a matchmaking loop that continuously checks for players in the queue, forms lobbies, and simulates matches.
- and finally a web-based dashboard: a next(react) frontend with shadcn/ui that connects to the server via WebSockets to display live matchmaking data, player stats, and game results.

## Components

1. **Server (`main.rs`)**: The core backend running async matchmaking loops, game simulations, and client state management.
2. **Dashboard (`useDashboard.ts`)**: A React-based web interface that connects to a dedicated WebSocket route (`/ws/dashboard`) to stream live server snapshots, queue info, and active games.
3. **Client Simulator**: A CLI application simulating players connecting, forming lobbies, and queuing.

## Getting Started (Linux)

This project uses [`just`](https://github.com/casey/just) as a command runner for convenience.

### Running the Services

Open separate terminal tabs and run the following commands:

```bash
# 1. Start the main server (binds to 127.0.0.1:12345 by default)
just run-server

# 2. Start the dashboard (frontend)
just run-dashboard

# 4. Start the simulation (simulation a bunch of players connecting, queuing, and playing matches)
just run-sim 

# 3. Start a manual client (you can run multiple instances to simulate multiple players)
just run-client

```

### Manual Testing with Websocat

You can interact with the WebSocket server directly using `websocat`, which is highly recommended for manual testing and inspecting raw JSON payloads.

```bash
# Connect as a new player
websocat ws://127.0.0.1:12345/ws/

# Or connect with a specific UUID
websocat ws://127.0.0.1:12345/ws/550e8400-e29b-41d4-a716-446655440000

```

Once connected, you can send raw JSON commands matching the `ClientMessage` enum, e.g.:
`{"type": "SearchGame"}`

Here are the available commands for the client:

| Command           | Description                                      |
|-------------------|--------------------------------------------------|
| `{"type": "JoinLobby"}` | Joins an existing lobby.                         |
| `{"type": "SearchGame"}` | Enters matchmaking queue.                        |
| `{"type": "CancelSearch"}` | Leaves matchmaking queue.                        |
| `{"type": "ConfirmGame", "id": "some-match-id"}` | Confirms a match when prompted. | 