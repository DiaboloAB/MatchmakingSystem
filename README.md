# Matchmaking Server & Simulation

A highly concurrent matchmaking system built with Rust, Axum, and Tokio. This project features a central game server, a dashboard for real-time monitoring, and a client simulator to test matchmaking logic and network resilience.

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

# 3. Start the client simulator to populate the server
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
