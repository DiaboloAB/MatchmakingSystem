run-server:
    RUST_LOG="debug" cargo run --package server
    
run-client:
    RUST_LOG="debug" cargo run -p client

run-sim:
    RUST_LOG="debug" cargo run --package sim
    