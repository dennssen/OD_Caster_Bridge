mod gui;
mod ws;

use std::sync::Arc;
use tokio::sync::RwLock;
use eframe::egui;
use crate::ws::{server, api};
use crate::gui::app::{OverlayProxyApp, AppState};
use crate::ws::state::GameState;

fn main() -> eframe::Result {
    let (broadcast_tx, _) = tokio::sync::broadcast::channel(100);

    let state = Arc::new(RwLock::new(AppState {
        game_state: GameState {
            game_data: None,
            gamemodes: vec![],
            selected_gamemode: None,
        },
        subscribed_gamemode_slot_id: String::new(),
        
        home_team_name: "Home Team".to_string(),
        away_team_name: "Away Team".to_string(),
        home_team_logo: None,
        away_team_logo: None,
        
        connected_clients: 0,
        spectator_connection: false,
        poll_interval_ms: 16,
        broadcast_tx,
    }));
    
    let game_data_poll_state = Arc::clone(&state);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            api::poll_game_data(game_data_poll_state).await;
        });
    });

    let ws_state = Arc::clone(&state);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            server::run_websocket_server(ws_state).await;
        });
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 620.0])
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };

    eframe::run_native(
        "Caster Bridge",
        options,
        Box::new(|_cc| Ok(Box::new(OverlayProxyApp::new(state)))),
    )
}
