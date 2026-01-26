mod gui;
mod ws;
mod managers;

use std::sync::Arc;
use tokio::sync::RwLock;
use eframe::egui;
use crate::ws::{server, api};
use crate::gui::app::{OverlayProxyApp, AppState};
use crate::managers::rounds::RoundManager;
use crate::ws::state::{CasterTeams, GameState};

fn main() -> eframe::Result {
    let (broadcast_tx, _) = tokio::sync::broadcast::channel(100);

    let state = Arc::new(RwLock::new(AppState {
        game_state: GameState {
            game_data: None,
            gamemodes: vec![],
            selected_gamemode: None,
            cameras: vec![],
            camera_api: None,
            caster_teams: CasterTeams::default(),
        },
        subscribed_gamemode_slot_id: String::new(),
        camera_api_id: "dennssen.caster".to_string(),
        
        round_manager: RoundManager::new(),
        
        connected_clients: 0,
        spectator_connection: false,
        poll_interval_fps: 60,
        poll_game_data: false,
        poll_gamemodes: true,
        poll_cameras: true,
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
            .with_inner_size([320.0, 780.0])
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };

    eframe::run_native(
        "Caster Bridge",
        options,
        Box::new(|cc| {
            cc.egui_ctx.style_mut(|style| {
                style.spacing.item_spacing.x = 2.0;
            });
            Ok(Box::new(OverlayProxyApp::new(state)))
        }),
    )
}
