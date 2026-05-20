# MatchFlow — Matchmaking System

A real-time matchmaking backend built with Rust, Axum, and Tokio. Designed as an ICT systems pipeline exercise focusing on data flow, degradation, and recovery across a live distributed system.

---

## Components

| Binary | Role |
|---|---|
| `server` | Central backend — matchmaking, game simulation, ranking, WebSocket hub |
| `client` | Manual single-player CLI — for testing and debugging |
| `sim` | Automated multi-player simulator — populates the server with realistic traffic |
| `dashboard` | Next.js web interface — real-time visualization of the full pipeline |

---

## Getting Started

### Prerequisites

- Rust (stable)
- Node.js 18+
- [`just`](https://github.com/casey/just) command runner

### Running the Services

Open separate terminal tabs:

```bash
# 1. Start the server (default port 12345)
just run-server

# 2. Start the dashboard (Next.js frontend)
just run-dashboard

# 3. Manual single-player client (for testing)
just run-client

# 4. Automated simulator (populates server with traffic)
just run-sim
```

---

## CLI Reference

### Server

```
Usage: server [OPTIONS]

Options:
  -p, --port <PORT>    Listening port [default: 12345]
  -h, --help           Print help
  -V, --version        Print version
```

### Client (manual single player)

```
Usage: client [OPTIONS]

Options:
      --port <PORT>    Server port [default: 12345]
      --host <HOST>    Server host [default: 127.0.0.1]
      --id <ID>        Player UUID — omit to create a new player
  -h, --help           Print help
  -V, --version        Print version
```

If `--id` is omitted a new UUID is generated and printed — copy it to reconnect as the same player and preserve your MMR.

### Simulator (automated multi-player)

```
Usage: sim [OPTIONS]

Options:
      --port <PORT>    Server port [default: 12345]
      --host <HOST>    Server host [default: 127.0.0.1]
  -c, --count <COUNT>  Number of simulated players [default: 50]
  -h, --help           Print help
  -V, --version        Print version
```

---

## Client Commands (manual client)

Once connected, type commands in the terminal:

| Command | Description |
|---|---|
| `JoinLobby` | Create a new lobby |
| `JoinLobby <uuid>` | Join an existing lobby by ID |
| `DisplayLobby` | Show current lobby state |
| `LeaveLobby` | Leave the current lobby |
| `SearchGame` | Start matchmaking for the lobby |
| `CancelSearch` | Cancel matchmaking |
| `ConfirmGame <uuid>` | Confirm a found game by game ID |
| `DisplayPlayer` | Show your player info |
| `DisplayPlayer <uuid>` | Show another player's info |
| `help` / `h` | Show help |
| `quit` / `q` | Disconnect |

---

## Player ↔ Server Communication

### Connection

Players connect via WebSocket. Two endpoints are available:

```
ws://<host>:<port>/ws/<player_uuid>   — reconnect as existing player
ws://<host>:<port>/ws                 — create a new player (UUID generated server-side)
```

On connection the server loads the player from SQLite (or creates a new entry), registers them in memory, and immediately sends a `Welcome` message.

### Player State Machine

Every player exists in exactly one state at all times. Transitions are **server-authoritative** — the client requests, the server decides.

```
Idle
 ├─► (JoinLobby)   → InLobby
 └─► (direct)      → InLobby

InLobby
 ├─► (SearchGame)  → InQueue
 └─► (LeaveLobby)  → Idle

InQueue
 ├─► (match found) → NeedConfirmation
 └─► (CancelSearch)→ InLobby

NeedConfirmation
 ├─► (ConfirmGame) → InGame
 └─► (timeout 15s) → InLobby (re-queued) or Idle (penalised)

InGame
 └─► (game ends)   → Idle
```

### Messages — Client → Server

```json
{ "type": "JoinLobby", "id": null }
{ "type": "JoinLobby", "id": "550e8400-e29b-41d4-a716-446655440000" }
{ "type": "DisplayLobby" }
{ "type": "LeaveLobby" }
{ "type": "SearchGame" }
{ "type": "CancelSearch" }
{ "type": "ConfirmGame", "id": "550e8400-e29b-41d4-a716-446655440000" }
{ "type": "DisplayPlayer", "id": null }
{ "type": "DisplayPlayer", "id": "550e8400-e29b-41d4-a716-446655440000" }
```

### Messages — Server → Client

**`Welcome`** — sent immediately on connection

```json
{
  "type": "Welcome",
  "player": {
    "id": "...", "name": "...", "mmr": 1000.0,
    "true_skill": 1000.0, "rank": "Silver",
    "wins": [], "losses": [], "status": "Idle"
  }
}
```

**`LobbyJoined`** — player successfully joined or created a lobby

```json
{ "type": "LobbyJoined", "lobby_id": "..." }
```

**`Lobby`** — current lobby state (response to DisplayLobby)

```json
{
  "type": "Lobby",
  "lobby_id": "...",
  "players": ["Alice", "Bob"],
  "status": "Idle"
}
```

**`LobbyLeft`** — player left the lobby

```json
{ "type": "LobbyLeft", "lobby_id": "..." }
```

**`SearchingGame`** — lobby entered the matchmaking queue

```json
{ "type": "SearchingGame", "lobby_id": "...", "player_searching": "Alice" }
```

**`SearchCancelled`** — matchmaking cancelled

```json
{ "type": "SearchCancelled", "lobby_id": "...", "player_cancelling": "Alice" }
```

**`GameFound`** — match found, player must confirm within 15 seconds

```json
{ "type": "GameFound", "game_id": "..." }
```

**`GameStarting`** — all players confirmed, game begins

```json
{ "type": "GameStarting", "game_id": "..." }
```

**`GameCancelled`** — someone did not confirm in time

```json
{ "type": "GameCancelled", "game_id": "..." }
```

**`GameResult`** — game finished, MMR updated

```json
{
  "type": "GameResult",
  "game_id": "...",
  "won": true,
  "mmr_change": 14.2,
  "new_mmr": 1014.2,
  "new_rank": "Silver"
}
```

**`PlayerUpdated`** — player data changed (after MMR update)

```json
{ "type": "PlayerUpdated", "player": { ... } }
```

**`Error`** — invalid action or state transition

```json
{ "type": "Error", "message": "Cannot join lobby while in queue" }
```

**`Info`** — informational message (e.g. help text)

```json
{ "type": "Info", "message": "Available commands: ..." }
```

---

## Dashboard ↔ Server Communication

### Connection

The dashboard connects to a dedicated read-mostly WebSocket endpoint:

```
ws://<host>:<port>/ws/dashboard
```

Unlike player connections, the dashboard can also send requests to fetch detailed data on demand.

### Automatic Snapshots

The server broadcasts a `DashboardSnapshot` every 500ms automatically — no request needed. The dashboard just receives and renders it.

```json
{
  "type": "Snapshot",
  "snapshot": {
    "total_player": 49,
    "connected_players": 12,
    "lobby_number": 4,
    "game_number": 3,
    "queueing_lobbies": 2,
    "waiting_games": 1,
    "ongoing_games": 2
  }
}
```

### Messages — Dashboard → Server

```json
{ "type": "GetPlayers" }
{ "type": "GetLobbys" }
{ "type": "GetGames" }
```

### Messages — Server → Dashboard

**`PlayerList`** — full list of connected players

```json
{
  "type": "PlayerList",
  "players": [
    {
      "id": "...", "name": "...", "mmr": 1024.0,
      "rank": 2, "div": 1, "status": "InGame",
      "skills": ["fire", "water", "thunder"]
    }
  ]
}
```

**`LobbyList`** — all active lobbies and queuing entries

```json
{
  "type": "LobbyList",
  "lobbys": [ { "id": "...", "players": ["..."], "status": "InQueue" } ],
  "queueing_lobby": [ { "lobby_id": "...", "avg_mmr": 1050.0, "player_count": 1 } ]
}
```

**`GameList`** — all active games

```json
{
  "type": "GameList",
  "waiting_games": [ { "game_id": "...", "player_a": "...", "player_b": "...", "status": "WaitingConfirmation" } ],
  "ongoing_games": [ { "game_id": "...", "player_a": "...", "player_b": "...", "status": "InProgress" } ]
}
```

---

## AppState — Shared Server State

All server state lives in `AppState`, wrapped in `Arc` so it can be shared cheaply across async tasks. Each field is behind a `RwLock` — multiple readers allowed, one writer at a time.

```rust
pub struct AppState {
    // live player data (connected players only)
    pub players:         Arc<RwLock<HashMap<Uuid, Player>>>,

    // one mpsc sender per connected client — how the server pushes messages
    pub senders:         Arc<RwLock<HashMap<Uuid, PlayerTx>>>,

    // active lobbies
    pub lobbys:          Arc<RwLock<HashMap<Uuid, Lobby>>>,

    // lobbies currently in the matchmaking queue (with precomputed avg_mmr)
    pub queueing_lobbys: Arc<RwLock<Vec<QueueEntry>>>,

    // games waiting for all players to confirm
    pub waiting_games:   Arc<RwLock<HashMap<Uuid, Game>>>,

    // games currently being simulated
    pub ongoing_games:   Arc<RwLock<HashMap<Uuid, Game>>>,

    // dashboard sender — broadcast channel, one per connected dashboard tab
    pub dashboard_tx:    Arc<RwLock<DashboardTx>>,

    // persistent storage — player MMR, rank, match history
    pub db:              SqlitePool,

    // server-wide settings (team size, simulation speed, etc.)
    pub settings:        Arc<RwLock<Settings>>,

    // total players ever created (persisted counter)
    pub total_player:    Arc<RwLock<u64>>,
}
```

### Key Design Decisions

**In-memory vs persistent** — live state (who is connected, who is queuing) lives in memory for speed. Only player rank/MMR and match history are written to SQLite, and only when a game resolves.

**`senders` map** — instead of holding a WebSocket reference per player, each connection creates an `mpsc` channel. The server stores the sender end; the connection task holds the receiver. This decouples the matchmaker from the WebSocket layer entirely — the matchmaker just drops a message into the channel without knowing anything about the connection.

**`queueing_lobbys` as `Vec<QueueEntry>`** — the matchmaker only needs `avg_mmr` and `queue_time` to make decisions. These are precomputed when a lobby calls `SearchGame`, so the matchmaker loop never touches the full player or lobby data during normal operation.

---

## Background Tasks

The server spawns four independent Tokio tasks on startup:

| Task | Interval | Responsibility |
|---|---|---|
| `matchmaking_loop` | 2s | Pairs queued lobbies by MMR proximity |
| `confirmation_loop` | 1s | Times out unconfirmed games, re-queues confirmed players |
| `game_simulation_loop` | 20s | Resolves finished games, updates MMR, saves to DB |
| `dashboard_broadcast_loop` | 500ms | Builds and sends `DashboardSnapshot` to dashboard |

All tasks share `AppState` via `Arc` clone — no message passing between tasks, no global variables.

---

## Ranking System

Two MMR values run in parallel per player:

| Field | Visibility | K-factor | Purpose |
|---|---|---|---|
| `mmr` | Visible (shown on dashboard) | 16 | Smooth rank progression the player sees |
| `true_skill` | Hidden (used by matchmaker) | 64 | Fast-converging estimate of real ability |

**Matchmaking** uses `true_skill` for fairer games from game one. **Rank display** uses `mmr` for smooth visible progression. After each game, the gap between the two is checked — if it exceeds ±300 points, the player is flagged:

| Gap | Flag |
|---|---|
| `true_skill - mmr > 300` | `smurf` — winning far more than rank suggests |
| `true_skill - mmr < -300` | `boosted` — losing far more than rank suggests |

Rank tiers are derived from `mmr` at display time:

| MMR | Rank |
|---|---|
| 0 – 799 | Iron |
| 800 – 999 | Bronze |
| 1000 – 1199 | Silver |
| 1200 – 1399 | Gold |
| 1400 – 1599 | Platinum |
| 1600 – 1799 | Diamond |
| 1800+ | Master |

---

## Database Schema

```sql
-- Player identity and rank (persistent across sessions)
CREATE TABLE players (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    mmr         REAL NOT NULL DEFAULT 1000.0,
    true_skill  REAL NOT NULL DEFAULT 1000.0,
    rank        INTEGER NOT NULL DEFAULT 1,
    div         INTEGER NOT NULL DEFAULT 1,
    anomaly     TEXT,  -- NULL, 'smurf', or 'boosted'
    skills      TEXT   -- comma-separated word list
);

-- One row per finished game
CREATE TABLE games (
    id          TEXT PRIMARY KEY,
    team1       TEXT NOT NULL,  -- comma-separated UUIDs
    team2       TEXT NOT NULL,
    winners     TEXT NOT NULL,
    mmr_change  REAL NOT NULL,
    played_at   TEXT NOT NULL   -- ISO 8601
);

-- One row per player per game (for fast history queries)
CREATE TABLE player_games (
    player_id   TEXT NOT NULL,
    game_id     TEXT NOT NULL,
    won         BOOLEAN NOT NULL,
    mmr_change  REAL NOT NULL,
    PRIMARY KEY (player_id, game_id)
);
```

---

## Network Resilience

Fault injection is implemented in the simulator rather than at the OS level (no `tc netem`). This gives precise control over which message is delayed or dropped, making it easier to isolate which state transition is being tested.

| Scenario | How it's simulated | Server recovery |
|---|---|---|
| Slow confirmation | Random delay 300ms–8s before `ConfirmGame` | 15s timeout, re-queue confirmed players |
| AFK on game found | 10% of players ignore `GameFound` entirely | Timeout triggers `GameCancelled` |
| Mid-session disconnect | Players disconnect after 1–8 games | Cleanup cascades: lobby → queue → game |
| Reconnection | Players reconnect after 5s–2min break | Server reloads player from DB |
| Idle timeout | No message for 30–120s → disconnect | Same cleanup as disconnect |