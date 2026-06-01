# Design Decisions & Troubleshooting

This document covers the major architectural choices made during development,
the reasoning behind them, and the technical problems encountered and solved.

## Part 1 — Architectural Decisions

### 1. Lobby System — Why Not Queue Players Directly

Early design queued players directly. Changed to lobby-centric queueing before
writing the matchmaker.

**Problem with player-centric queue:** adding multi-player teams later would require
rewriting the matchmaker entirely — it would need to understand "groups" as a new concept.

**Lobby-centric solution:** the queue holds `QueueEntry` structs (lobby ID + precomputed
avg MMR). The matchmaker pairs lobbies, not players. A lobby with 1 player and a lobby
with 5 players look identical to the matchmaker — it just pairs two of them.

```
now:    Lobby { players: [A] }         → matchmaker sees one QueueEntry
future: Lobby { players: [A, B, C] }   → matchmaker sees one QueueEntry
```

Zero matchmaker changes needed to support team play.

### 2. Game Simulation — From Real Game to Skill Values

**What was tried first:** implement an actual simple game (turn-based word battle,
players pick words with hidden power values, best of 5 rounds).

**Why it was simplified:** the game mechanic added complexity with no system design
value. The interesting part of the project is the pipeline around the game, not the
game itself. Time spent on game rules is time not spent on matchmaking, recovery, or
the dashboard.

**Final approach:** each player has two simulation-only fields:
- `debug_player_level` (0.0–10.0) — base skill
- `debug_player_form` (0.8–1.2) — multiplier simulating a good or bad day

Match outcome:

```rust
t1_score = sum of (player.level * player.form + random noise) for team 1
t2_score = sum of (player.level * player.form + random noise) for team 2
winner   = higher score
```

This produces realistic variance — a stronger player wins more often but not always.
The random noise ensures upsets happen, which makes the MMR system actually do work
over time rather than converging instantly.

### 3. Dual MMR — Visible vs Hidden

A single MMR value has a problem: if it moves fast, the displayed rank is unstable
and players see their rank yo-yoing. If it moves slow, smurfs take hundreds of games
to reach their real rank.

Real competitive games (League of Legends, TrueSkill) solve this with two values:

| Field | K-factor | Purpose |
|---|---|---|
| `mmr` (visible) | 16 — slow | What the player sees. Smooth, stable progression. |
| `true_skill` (hidden) | 64 — fast | What the matchmaker uses. Converges to real skill quickly. |

The gap between the two is the anomaly detection signal:
- `true_skill - mmr > 300` → player is winning far above their displayed rank → **smurf**
- `true_skill - mmr < -300` → player is losing far below their displayed rank → **boosted**

This is the AI-based processing layer of the pipeline. Framed honestly as statistical
anomaly detection, not a trained model — appropriate for the data volume available.

### 4. rusqlite → sqlx

Original database code used `rusqlite` (synchronous SQLite bindings).

**Problem discovered:** calling a synchronous function inside a Tokio async task
blocks the entire thread. With many concurrent connections, one slow DB write could
stall unrelated tasks — exactly the kind of latency spike visible on the dashboard.

**Fix:** replaced with `sqlx`, which is async-native. DB calls use `.await` normally
and Tokio can schedule other work while waiting for the disk. Same SQLite file,
same SQL queries, zero data migration needed.

### 5. Network Constraint Approach — Application Level vs OS Level

Two options for simulating network problems:

**OS level (`tc netem`):**
```bash
sudo tc qdisc add dev lo root netem delay 200ms loss 10%
```
Affects all traffic on the loopback interface indiscriminately. Good for realistic
end-to-end testing. Hard to target specific message types.

**Application level (in the simulator):**
```rust
async fn send_with_constraints(ws_tx, msg, loss_rate_percent: 10, max_latency_ms: 2000)
```
Targets specific messages (e.g., only delay `ConfirmGame` packets). Easy to isolate
which state transition is being tested. Easier to debug when something breaks.

**Decision:** implement both. Application-level for targeted fault injection during
development. `tc netem` for the live demo to show real network degradation with no
code changes.

**Important nuance:** WebSocket runs over TCP. `tc netem` packet loss does not drop
messages — TCP retransmits them. The observable effect is increased latency, which
causes confirmation timeouts to fire. This is actually the interesting behavior:
showing that application-level timeouts handle TCP-level degradation correctly.

## Part 2 — Technical Problems & Solutions

### 1. Async Deadlocks (`RwLock` Blocks)

**Symptom:** Server stops processing. New players can't connect. Process hasn't crashed.

**Cause:** Holding a `tokio::sync::RwLock` across an `.await` point, or acquiring
multiple locks in different orders across different tasks.

```rust
// WRONG — deadlock risk
let mut games = state.waiting_games.write().await;
state.ongoing_games.write().await.insert(...); // hangs trying to acquire second lock
```

**The three rules established:**

1. **Snapshot, Drop, Act** — collect what you need, drop the lock, then act.
2. **Single Lock Policy** — never hold more than one lock simultaneously.
3. **Explicit Scoping** — use `{}` blocks to guarantee drop before any `.await`.

```rust
// CORRECT
let confirmed: Vec<(Uuid, Game)> = {
    let games = state.waiting_games.read().await;
    games.iter().filter(...).map(|(id, g)| (*id, g.clone())).collect()
}; // lock dropped here — safe to acquire other locks below
```

### 2. Ghost Players (State Leak on Disconnect)

**Symptom:** Dashboard shows players in lobby or queue that have actually disconnected.
Matchmaker tries to send messages to dead channels.

**Cause:** Cleanup on disconnect only removed the player from `state.players` and
`state.senders`, leaving their UUID inside lobby player lists and the matchmaking queue.

**Solution:** Disconnection cascades through all state:

```
player disconnects
    └── remove from state.players + state.senders
    └── find their lobby → remove from lobby.players
        └── if lobby empty → delete lobby
        └── if lobby was queuing → remove from queueing_lobbys
        └── if lobby in waiting_game → cancel game, re-queue other players
```

### 3. Wrong Bind Order in sqlx (Silent Data Corruption)

**Symptom:** Players saved to DB but MMR always 0, ranks wrong, wins/losses corrupted.

**Cause:** `sqlx` binds map positionally to `?` placeholders. One missing `.bind()`
call shifts all subsequent values into the wrong columns. The error was silenced by
`let _ = query.execute(db).await` which discards the Result entirely.

```rust
// WRONG — true_skill bind was missing, everything after shifted
.bind(player.mmr)
// .bind(player.true_skill)  ← missing
.bind(wins_json)             // went into true_skill column
.bind(losses_json)           // went into wins column
```

**Fix:** add the missing bind, and replace `let _ =` with proper error logging:

```rust
if let Err(e) = query.execute(db).await {
    log::error!("Failed to save player {}: {}", player.id, e);
}
```

**Lesson:** never silently discard DB errors during development.

### 4. `rand::rng()` Not `Send` (Simulator Compile Error)

**Symptom:**
```
future cannot be sent between threads safely
the trait `Send` is not implemented for `Rc<UnsafeCell<BlockRng<...>>>`
```

**Cause:** `rand::rng()` returns a thread-local RNG that is not `Send`. Storing it
as a variable in an `async fn` that gets passed to `tokio::spawn` fails because
Tokio requires spawned futures to be `Send`.

**Fix:** use `SmallRng` which is `Send`:

```rust
use rand::rngs::SmallRng;
use rand::SeedableRng;

let mut rng = SmallRng::from_os_rng(); // Send-safe, one per task
```

### 5. Simulator Bots Stuck After Timeout

**Symptom:** After a confirmation timeout, the server sends `StatusUpdate { status: "Idle" }`
to re-queued players. Simulator bots ignored this message and stayed stuck — never
searching for a game again.

**Cause:** The simulator's message handler had no case for `StatusUpdate`.

**Fix:** add the handler:

```rust
"StatusUpdate" => {
    if msg.get("status").and_then(|s| s.as_str()) == Some("Idle") {
        sleep(Duration::from_millis(rand::random_range(1000..3000))).await;
        send(ws_tx, json!({ "type": "SearchGame" })).await;
    }
}
```

**Secondary fix:** added a 60-second watchdog timeout around `ws_rx.next()` so bots
reconnect automatically if they stop receiving messages for any reason.

### 6. Dashboard Snapshot Too complex

There is a lot of data to show on the dashboard, and a lot of information to track. Since I'm sending raw json of my structs (players, lobbies) to the dashboard, im sending a lot of data. I would need to send batch of data and only send valuable information, but for simplicity, I just send everything once on connection. But I update frequently the main information through a snapshot every 500ms.
