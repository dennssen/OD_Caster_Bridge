mod gui;
mod ws;

use std::sync::Arc;
use tokio::sync::RwLock;
use eframe::egui;
use crate::ws::server;
use crate::gui::app::{OverlayProxyApp, AppState};

fn main() -> eframe::Result {
    let state = Arc::new(RwLock::new(AppState {
        game_state: None,
        home_team_name: "Home Team".to_string(),
        away_team_name: "Away Team".to_string(),
        home_team_logo: None,
        away_team_logo: None,
        connected_clients: 0,
        poll_interval_ms: 100,
    }));

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
        Box::new(|_cc| {
            Ok(Box::new(OverlayProxyApp::new(state)))
        }),
    )
}
