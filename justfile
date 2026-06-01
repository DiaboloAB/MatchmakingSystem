run-server:
    RUST_LOG=debug,sqlx=warn cargo run -p server

run-ws:
    websocat ws://127.0.0.1:12345/ws_new/

run-dashboard:
    cd ict-dashboard && npm run dev

run-client:
    RUST_LOG="debug" cargo run -p client

run-sim:
    RUST_LOG="debug" cargo run -p sim

run-sim-loss:
    RUST_LOG="debug" cargo run -p sim -- --packet-loss