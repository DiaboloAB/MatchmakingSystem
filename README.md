# Matchmaking Server & Simulation

This project is a matchmaking system built with Rust, Axum, and Tokio. Developed as a System Design project, it focuses on how data flows, degrades, and is recovered across a distributed system.

This project features a central matchmaking server, a real-time monitoring dashboard, and a multi-player simulator to test matchmaking logic and network resilience under intentional constraints.

## Project Scope & Requirements

This project maps to the following system design requirements:

- **Data Generation**: A dedicated simulator spawns N players generating continuous state events.
- **Transmission + Constraint**: Real-time WebSockets with fault injection (AFK, delays, disconnects, and OS-level network degradation).
- **Data Collection**: The server collects all player, match, and lobby events in memory and persists history to SQLite.
- **AI-based Processing**: Dual MMR anomaly detection (identifying "smurf" or "boosted" accounts).
- **Decision / Action**: Automated match formation, MMR updates, penalties, and re-queuing.
- **Visualization**: A Next.js dashboard providing a live feed of the server state via WebSockets.

## Components

1.  **✅Server (`/server`)**: The core engine running async matchmaking loops, game simulations, and client state management.
2.  **✅Dashboard (`/ict-dashboard`)**: A React-based web interface (`/ws/dashboard`) for visualizing live queue info, active games, and player stats.
3.  **❌ Data (`/data`)**: There is no static data; the data is generated on the fly by the simulator.
4.  **Simulator (`/sim`)**: An automated multi-player CLI that simulates realistic player behavior, including session churn and network issues.
5.  **Client (`/client`)**: A manual CLI application for debugging individual player connections and state transitions.

## Tech Stack

| Technology | Role |
| :--- | :--- |
| **Rust** | Server, Simulator, and Client core |
| **Tokio** | Asynchronous runtime |
| **Axum** | HTTP & WebSocket server |
| **sqlx + SQLite** | Asynchronous persistence |
| **React + Next.js** | Web dashboard |
| **shadcn/ui** | UI Components |

## Getting Started (Linux)

This project uses [`just`](https://github.com/casey/just) as a command runner.

### Running the Services

Open separate terminal tabs and run:

```bash
# 1. Start the main server (127.0.0.1:12345)
just run-server

# 2. Start the dashboard
just run-dashboard

# 3. Start the simulation (spawns automated players)
just run-sim

# 4. Start a manual client (for debugging)
just run-client
```

### Manual Testing

You can interact with the WebSocket server directly using `websocat`:

```bash
# Connect as a new player
websocat ws://127.0.0.1:12345/ws/
```

Send JSON commands: `{"type": "SearchGame"}`, `{"type": "JoinLobby"}`, etc.

## Documentation

- [**ARCHITECTURE.md**](./ARCHITECTURE.md): Deep dive into the system design, state machine, and ranking algorithm.
- [**TROOBLESHOOT.md**](./TROOBLESHOOT.md): Common issues like async deadlocks and state management.
- [**DATA.md**](./DATA.md): Information regarding data generation and the lack of static data samples.
 
## Use of Generative AI Tools

Generative AI tools were used in this project for:
- **Documentation**: Structuring my documentation and my idea to ensure clarity.
- **Code Review**: Identifying potential issues in async code patterns and suggesting improvements.
- **Simulation Script**: Generating a testing script (`/sim`) to simulate player connections and behaviors.
- **Dashboard components**: Some components such as the ranking repartition graph, or other table were created with the help of AI.

Most of the rust code and the architecture was designed and implemented by myself.

## Some Sources and Inspiration

- [sqlx tutorial](https://aarambhdevhub.medium.com/rust-orms-in-2026-diesel-vs-sqlx-vs-seaorm-vs-rusqlite-which-one-should-you-actually-use-706d0fe912f3)
- [websocket with axum](https://websocket.org/guides/languages/rust/)
- [riot games tech blogs](https://technology.riotgames.com/tags/infrastructure)

### librairies
- [random_word](https://crates.io/crates/random_word)
- [sqlx](https://crates.io/crates/sqlx)
- [tokio](https://crates.io/crates/tokio)
- [clap](https://crates.io/crates/clap)
- [uuid](https://crates.io/crates/uuid)
- [log](https://crates.io/crates/log)
- [env_logger](https://crates.io/crates/env_logger)
- [serde](https://crates.io/crates/serde)
- [axum](https://crates.io/crates/axum)
- [futures](https://crates.io/crates/futures)
- [serde_json](https://crates.io/crates/serde_json)
- [names](https://crates.io/crates/names)
- [rand](https://crates.io/crates/rand)

## Video presentation

[You can watch the video presentation of this project here](https://youtu.be/qASIMoXsjVA)