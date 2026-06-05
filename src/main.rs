#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod gui;
mod ws;
mod managers;
mod http;

use std::sync::Arc;
use tokio::sync::RwLock;
use eframe::egui;
use egui_extras::install_image_loaders;
use crate::ws::{server, api};
use crate::gui::app::{GUIData, AppState};
use crate::managers::appdata::AppData;
use crate::managers::rounds::RoundManager;
use crate::ws::state::{CameraApi, CasterTeams, GameState, GamemodeData, MatchData};

fn main() -> eframe::Result {
    let app_data = AppData::get_or_init();

    let (broadcast_tx, _) = tokio::sync::broadcast::channel(100);

    let state = Arc::new(RwLock::new(AppState {
        game_state: GameState {
            game_data: None,
            gamemodes: vec![],
            selected_gamemode: {
                #[cfg(debug_assertions)]
                {
                    Some(GamemodeData::default())
                }
                #[cfg(not(debug_assertions))]
                {
                    None
                }
            },
            cameras: vec![],
            camera_api: {
                #[cfg(debug_assertions)]
                {
                    Some(CameraApi::default())
                }
                #[cfg(not(debug_assertions))]
                {
                    None
                }
            },
            caster_teams: CasterTeams::default(),
            match_data: MatchData::default(),
        },
        subscribed_gamemode_slot_id: String::new(),
        camera_api_id: app_data.camera_api_id,
        
        round_manager: RoundManager::new(),
        selected_round: None,
        
        connected_clients: 0,
        spectator_connection: false,
        poll_interval_fps: app_data.poll_interval_fps,
        poll_game_data: app_data.poll_game_data,
        poll_gamemodes: app_data.poll_gamemodes,
        poll_cameras: app_data.poll_cameras,
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

    std::thread::spawn(|| {
        http::server::handle_http();
    });

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([340.0, 780.0])
        .with_resizable(false)
        .with_maximize_button(false);

    if let (Some(pos_x), Some(pos_y)) = (app_data.window_position_x, app_data.window_position_y) {
        viewport = viewport.with_position([pos_x, pos_y]);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Caster Bridge",
        options,
        Box::new(|cc| {
            cc.egui_ctx.style_mut(|style| {
                style.spacing.item_spacing.x = 2.0;
            });
            install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(GUIData::new(state)))
        }),
    )
}
