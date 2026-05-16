// use futures_util::{SinkExt, StreamExt};
use rand::{Rng, RngExt};
use std::time::Duration;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

async fn connect_with_backoff(url: &str) {
    let mut delay = Duration::from_secs(1);

    loop {
        match connect_async(url).await {
            Ok((ws, _)) => {
                loop {}
                // delay = Duration::from_secs(1);
                // let (mut sink, mut source) = ws.split();
                // let _ = sink.send(Message::text("hello")).await;

                // while let Some(Ok(msg)) = source.next().await {
                //     eprintln!("received: {msg}");
                // }
                // eprintln!("disconnected, reconnecting...");
            }
            Err(e) => eprintln!("connect failed: {e}"),
        }
        // Jitter prevents all clients reconnecting at once
        let jitter = rand::rng().random_range(0..500);
        let wait = delay + Duration::from_millis(jitter);
        tokio::time::sleep(wait).await;
        delay = (delay * 2).min(Duration::from_secs(30));
    }
}

fn main() {
    let url = "ws://localhost:12345";
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(connect_with_backoff(url));
}
