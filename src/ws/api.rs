use std::sync::Arc;
use tokio::sync::RwLock;
use crate::gui::app::AppState;
use crate::ws::state::GameState;

pub async fn poll_spec_api(state: Arc<RwLock<AppState>>) {
let client = reqwest::Client::new();

    loop {
        let interval = {
            let s = state.read().await;
            s.poll_interval_ms
        };

        match client.get("http://localhost:5420/state/")
            .send()
            .await
        {
            Ok(response) => {
                if let Ok(game_state) = response.json::<GameState>().await {
                    println!("[API] Fetched game_state");
                    {
                        let mut s = state.write().await;
                        s.game_state = Some(game_state.clone());
                    }
                    
                    let _ = state.read().await.broadcast_tx.send(game_state);
                }
            }
            Err(e) => {
                eprintln!("Failed to poll spectator API: {}", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(interval)).await;
    }
}