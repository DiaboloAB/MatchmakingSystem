use crate::app_state::AppState;

pub async fn matchmaking_loop(state: AppState) {
    log::info!("Matchmaker started");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        // try_form_matches(&state).await;
    }
}
