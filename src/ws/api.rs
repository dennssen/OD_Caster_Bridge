use std::sync::Arc;
use serde::Deserialize;
use tokio::sync::RwLock;
use crate::gui::app::AppState;
use crate::ws::state::{GameData, GamemodeData, SimpleGamemode};

#[derive(Clone, Deserialize)]
struct GamemodesResponse {
    gamemodes: Vec<SimpleGamemode>,
}

pub async fn poll_game_data(state: Arc<RwLock<AppState>>) {
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
                if let Ok(game_data) = response.json::<GameData>().await {
                    {
                        let mut s = state.write().await;
                        s.spectator_connection = true;
                        s.game_state.game_data = Some(game_data.clone());
                    }
                }
            }
            Err(e) => {
                let mut s = state.write().await;
                s.spectator_connection = false;
                eprintln!("Failed to poll spectator API: {}", e);
            }
        }

        match client.get("http://localhost:5420/state/gamemodes")
            .send()
            .await
        {
            Ok(response) => {
                if let Ok(response_data) = response.json::<GamemodesResponse>().await {
                    {
                        let mut s = state.write().await;
                        s.spectator_connection = true;

                        for gamemode in &response_data.gamemodes {
                            if gamemode.slot_id == s.subscribed_gamemode_slot_id {
                                match client.get(format!("http://localhost:5420/state/gamemodes/{}", gamemode.slot_id))
                                    .send()
                                    .await
                                {
                                    Ok(gamemode_data_response) => {
                                        if let Ok(gamemode_data) = gamemode_data_response.json::<GamemodeData>().await {
                                            s.game_state.selected_gamemode = Some(gamemode_data);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to get gamemode: {}: {}", gamemode.slot_id, e);
                                    }
                                }
                            }
                        }

                        s.game_state.gamemodes = response_data.gamemodes;
                    }
                }
            }
            Err(e) => {
                let mut s = state.write().await;
                s.spectator_connection = false;
                eprintln!("Failed to poll spectator API: {}", e);
            }
        }



        {
            let s = state.read().await;
            let _ = s.broadcast_tx.send(s.game_state.clone());
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(interval)).await;
    }
}