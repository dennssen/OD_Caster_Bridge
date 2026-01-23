use std::sync::Arc;
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::RwLock;
use crate::gui::app::AppState;
use crate::ws::state::{CameraApi, GameData, GamemodeData, SimpleGamemode};

#[derive(Clone, Deserialize)]
struct GamemodesResponse {
    gamemodes: Vec<SimpleGamemode>,
}

#[derive(Clone, Deserialize)]
struct CamerasResponse {
    cameras: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct CameraConfigResponse {
    api: CameraApi
}

pub async fn poll_game_data(state: Arc<RwLock<AppState>>) {
    let client = Client::new();

    loop {

        let s = state.read().await;

        let interval = 1000 / s.poll_interval_fps;
        let poll_game_data = s.poll_game_data;
        let poll_gamemodes = s.poll_gamemodes;
        let poll_cameras = s.poll_cameras;

        drop(s);

        if poll_game_data {
            handle_game_data(&client, &state).await;
        }

        if poll_gamemodes {
            handle_gamemodes(&client, &state).await;
        }

        if poll_cameras {
            handle_cameras(&client, &state).await;
        }

        {
            let s = state.read().await;
            let _ = s.broadcast_tx.send(s.game_state.clone());
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(interval)).await;
    }
}

async fn handle_game_data(client: &Client, state: &Arc<RwLock<AppState>>) {
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
}

async fn handle_gamemodes(client: &Client, state: &Arc<RwLock<AppState>>) {
    match client.get("http://localhost:5420/state/gamemodes")
        .send()
        .await
    {
        Ok(response) => {
            if let Ok(response_data) = response.json::<GamemodesResponse>().await {
                handle_subscribed_gamemode(&response_data, &client, state).await;

                let mut s = state.write().await;
                s.game_state.gamemodes = response_data.gamemodes;
            }
        }
        Err(e) => {
            eprintln!("Failed to poll spectator API: {}", e);
        }
    }
}

async fn handle_subscribed_gamemode(response_data: &GamemodesResponse, client: &Client, state: &Arc<RwLock<AppState>>) {
    let subscribed_slot_id = {
        let s = state.read().await;
        s.subscribed_gamemode_slot_id.clone()
    };
    let is_subscribed = response_data.gamemodes
        .iter()
        .any(|gm| gm.slot_id == subscribed_slot_id);

    if !is_subscribed {
        return
    }

    match client.get(format!("http://localhost:5420/state/gamemodes/{}", subscribed_slot_id))
        .send()
        .await
    {
        Ok(gamemode_data_response) => {
            if let Ok(gamemode_data) = gamemode_data_response.json::<GamemodeData>().await {
                let mut s = state.write().await;
                s.game_state.selected_gamemode = Some(gamemode_data);
            }
        }
        Err(e) => {
            eprintln!("Failed to get gamemode: {}: {}", subscribed_slot_id, e);
        }
    }
}

async fn handle_cameras(client: &Client, state: &Arc<RwLock<AppState>>) {
    match client.get("http://localhost:5420/cameras")
        .send()
        .await
    {
        Ok(response) => {
            if let Ok(cameras_data) = response.json::<CamerasResponse>().await {
                handle_subscribed_config(&cameras_data, &client, state).await;

                let mut s = state.write().await;
                s.game_state.cameras = cameras_data.cameras;
            }
        }
        Err(e) => {
            eprintln!("Failed to get cameras: {}", e);
        }
    }
}

async fn handle_subscribed_config(cameras_response: &CamerasResponse, client: &Client, state: &Arc<RwLock<AppState>>) {
    let camera_api_id = {
        let s = state.read().await;
        s.camera_api_id.clone()
    };

    let is_subscribed = cameras_response.cameras
        .iter()
        .any(|cam| *cam == camera_api_id);

    if !is_subscribed {
        return;
    }

    match client.get(format!("http://localhost:5420/cameras/{}/config", camera_api_id))
        .send()
        .await
    {
        Ok(response) => {
            if let Ok(camera_config) = response.json::<CameraConfigResponse>().await {
                let mut s = state.write().await;
                s.game_state.camera_api = Some(camera_config.api);
            }
        }
        Err(e) => {
            eprintln!("Failed to get subscribed camera: {}", e);
        }
    }
}