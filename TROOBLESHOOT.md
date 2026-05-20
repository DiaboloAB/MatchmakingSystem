# Troubleshooting & Known Issues

Building a highly concurrent state machine over WebSockets introduces specific challenges. Below are common issues and how they are handled (or how to debug them).

## 1. Async Deadlocks (`RwLock` Blocks)

**Symptom:** The server stops processing matchmaking, or new players can't connect, but the process hasn't crashed.

**Cause:** This is almost always an async deadlock caused by holding a `tokio::sync::RwLock` across an `await` point, or acquiring multiple locks in different orders.
For example:
```rust
// BAD: Holding the lock across an await point
let mut lobbys = state.lobbys.write().await;
do_something_async().await; // Code blocks here, nobody else can access lobbys!

```

**Solution:** Always drop the lock before awaiting. Use explicit scoping:

```rust
// GOOD: Scope the lock
let lobby_ids = {
    let lobbys = state.lobbys.read().await;
    lobbys.keys().cloned().collect::<Vec<_>>()
}; // Lock is dropped here!
// Now safe to await
state.send_to_players(lobby_ids, msg).await; 

```

## 2. Ghost Players (State Management Disconnects)

**Symptom:** The dashboard shows players in a lobby or queue, but those players have actually disconnected.

**Cause:** A player closed their socket (or their connection dropped) while they were deep in a state machine flow (e.g., inside a Lobby, or during Game Confirmation). If the cleanup function only removes them from `state.players` but leaves their UUID inside `state.lobbys` or `state.waiting_games`, the matchmaking loop will panic or hang trying to route messages to a dead channel.

**Solution:** Disconnection logic must cascade. When `ws_rx.next()` terminates, the cleanup routine must:

1. Check if the player is in a lobby, and remove them. If the lobby is now empty, delete the lobby.
2. If the lobby was in `queueing_lobbys`, remove it from the queue.
3. If the player was in a `NeedConfirmation` state, fail the game confirmation and return the other players to the queue.
4. Finally, remove the player from `state.players` and `state.senders`.

## 3. SQLite Database Locked

**Symptom:** `sqlx` throws a `database is locked` error.

**Cause:** SQLite allows concurrent reads but only one concurrent write. If multiple asynchronous tasks attempt to write simultaneously (e.g., saving MMR for 50 players the exact moment a match ends) without proper connection pooling, it locks.

**Solution:** Ensure `mode=rwc` is used and consider tweaking the `SqlitePoolOptions` to increase the `max_connections` or `idle_timeout`. Batching database updates (saving match results in one transaction) rather than per-player saves also mitigates this.
