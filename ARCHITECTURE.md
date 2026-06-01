# Architecture

This project is a state-driven matchmaking system. The architecture prioritizes performance, memory safety, and the decoupling of the matchmaking engine from the network layer.

## System Overview

The system is composed of four main pillars:
1.  **AppState**: The "Shared Brain" of the server, managing live state in memory.
2.  **Matchmaking Engine**: Independent loops governing game formation and simulation.
3.  **WebSocket Layer**: A communication hub using the Sender Channel pattern.
4.  **Persistence Layer**: Asynchronous SQLite for historical data.

## 1. AppState & State Management

A central `AppState` is shared across all asynchronous tasks via `Arc<RwLock<T>>`.

### Live vs. Persistent State
-   **Live State**: Information about connected players, active lobbies, and ongoing games lives entirely in memory for high-speed access.
-   **Persistent State**: Player MMR, ranks, and match history are stored in SQLite using `sqlx`. Updates occur only when a game resolves, mimicking production patterns (e.g., Redis for live state, PostgreSQL for history).

### The Sender Channel Pattern
To decouple the matchmaking logic from WebSocket handling, every connected player is assigned an `mpsc::unbounded_channel`.
-   The **Matchmaker** drops messages into the player's sender channel.
-   The **Connection Task** listens to the receiver end and forwards messages to the WebSocket.
This ensures that the matchmaker never holds direct WebSocket references, preventing complex ownership issues.

## 2. Background Loops

Four independent loops run concurrently, sharing the `AppState`:

| Task | Interval | Responsibility |
| :--- | :--- | :--- |
| **Matchmaking** | 2s | Pairs queued lobbies based on MMR and wait time. |
| **Confirmation** | 1s | Handles timeouts for games awaiting player confirmation. |
| **Simulation** | 20s | Resolves finished games, calculates word battle results, and updates MMR. |
| **Dashboard** | 500ms | Broadcasts a full state snapshot to the monitoring dashboard. |

## 3. Player Lifecycle & State Machine

The server is the ultimate authority. Clients request state transitions, and the server validates and executes them.

**Lifecycle:**
`Idle → InLobby → InQueue → NeedConfirmation → InGame → Idle`

### Lobby-Centric Design
The matchmaking queue holds **Lobby IDs**, not Player IDs. This design allows for future expansion to multi-player teams (e.g., 5v5) without changing the core matchmaking logic.

## 4. Ranking System: Dual MMR

The system uses a dual-value ranking system to balance matchmaking speed with visible progression.

| Value | K-Factor | Visibility | Purpose |
| :--- | :--- | :--- | :--- |
| **MMR** | 16 (Slow) | Visible | Smooth visible progression for the player. |
| **True Skill** | 64 (Fast) | Hidden | High-volatility value used for fast matchmaking convergence. |

### Anomaly Detection (AI Layer)
No AI is implemented, but the architecture supports future integration of an anomaly detection layer that flags accounts with suspicious MMR trajectories (e.g., smurf or boosted accounts).

## 5. Game Simulation: Skill + Randomness

To simulate games without complex gameplay code, the system attach two value to simulate the skill of the player: Level for the level ceiling of the player and Form for the temporal consistency of the player. Then the winner is just decided with a simple random formula on these two values. We focus on the matchmaking and state management.

## 7. Communication Protocols

MatchFlow uses WebSockets for all real-time interactions. There are two different "languages" (protocols) used in the system:

### A. Player ↔ Server (Event-Driven)
The communication between a player and the server is **Event-Driven**. This means messages are sent only when something happens.
- **Client Messages**: Actions like `JoinLobby`, `SearchGame`, or `ConfirmGame`.
- **Server Messages**: Notifications like `GameFound`, `StatusUpdate`, or `GameResult`.

This protocol ensures that the player's app only reacts to specific changes, saving bandwidth and processing power.

### B. Dashboard ↔ Server (Snapshot Pattern)
The Dashboard uses a **Snapshot Pattern**. To keep the monitoring simple and always accurate:
- Every **500ms**, the server sends a "Snapshot"—a complete summary of the current state (number of players, active games, queue length).
- The Dashboard simply replaces its old data with the new Snapshot.
- If the Dashboard needs more detail (like a full list of players), it can request it specifically using `GetPlayers` or `GetGames`.

## 8. Database Schema

MatchFlow uses **SQLite** to remember information even after the server restarts. The data is organized into three main tables:

| Table | Purpose |
| :--- | :--- |
| **`players`** | Stores identity (Name, UUID), Rank, and performance metrics (Wins, Losses, True Skill). |
| **`games`** | Stores a history of every finished match, including which teams played and who won. |
| **`player_games`** | A "join table" that connects players to their match history for fast lookups. |

*Note: In-memory data (like who is currently in a lobby) is never saved to the database. Only "final" results like MMR changes are persisted.*

## 9. Key Design: The "Sender Channel"

One of the most important parts of the code is how the server talks to players without getting "confused" by slow internet.

We use a **Sender Channel** pattern:
1. Every player gets a private "mailbox" (an asynchronous channel) when they connect.
2. The Matchmaker drops a message into the mailbox and immediately moves on to the next task.
3. A separate background task picks up the message and sends it over the internet.

This prevents the whole server from slowing down if one player has a bad connection.

The system is still in developpment, and lack of protection against edge cases, ex: if a lobby cancel a queue while the confirmation time, the others lobby are not notified and are stuck till the confirmation timeout.
