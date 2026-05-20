# System Architecture

This matchmaking system is designed around an asynchronous, event-driven architecture using **Rust** and **Tokio**.

## Core State (`AppState`)

The entire server state is held within `AppState`, passed to handlers and loops via Axum's state management. Data is guarded by `Arc<RwLock<T>>` to allow concurrent reads and exclusive writes. 

* **Players & Lobbies:** HashMaps track connected players and active lobbies.
* **Queues & Games:** Data structures manage users waiting for games, requiring confirmation, or actively playing.
* **Database:** `SqlitePool` handles persistent records (MMR, Win/Loss, Skills).

## Async Control Loops

The server spawns several independent Tokio tasks on startup:
1. `matchmaking_loop`: Continuously scans the `queueing_lobbys` to group players based on MMR and Settings.
2. `confirmation_loop`: Tracks games pending player confirmation. Handles timeouts.
3. `game_simulation_loop`: Simulates match duration and resolves outcomes.
4. `dashboard_broadcast_loop`: Aggregates the `AppState` into a `DashboardSnapshot` and broadcasts it to connected UI clients.

## Network Constraints & Resilience Testing

To test system resilience, we inject **intentional network constraints**. 

### Application-Level vs. OS-Level Constraints

While Linux provides powerful OS-level networking tools like `tc` (Traffic Control) and `netem` to simulate lag, packet loss, or jitter, we chose to implement **Application-Level Simulation** inside the Client.

**Why?**
1. **Targeted Debugging:** Using `tc/netem` drops packets indiscriminately across the whole port. If the server crashes, it's hard to isolate *which* state transition failed.
2. **Feature-Specific Simulation:** By implementing constraints in the client simulator, we can inject logic like:
   * *Delay only the `ConfirmGame` packet* to test the server's timeout logic.
   * *Drop the `GameResult` acknowledgment* to see if the server cleans up the lobby properly.
3. **Granular Control:** We can configure "Player A" to have 500ms lag, while "Player B" drops 10% of their packets, simulating real-world mixed-network scenarios.

This architecture ensures our server's core logic is heavily tested against edge cases without obfuscating the source of the errors.
