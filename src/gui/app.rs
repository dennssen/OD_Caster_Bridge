use std::sync::Arc;
use tokio::sync::RwLock;
use eframe::egui;
use crate::gui::widgets;
use crate::ws::state::GameState;

pub struct AppState {
    pub game_state: Option<GameState>,
    pub home_team_name: String,
    pub away_team_name: String,
    pub home_team_logo: Option<String>,
    pub away_team_logo: Option<String>,
    pub connected_clients: usize,
    pub poll_interval_ms: u64,
    pub broadcast_tx: tokio::sync::broadcast::Sender<GameState>,
}

pub struct OverlayProxyApp {
    state: Arc<RwLock<AppState>>,
}

impl OverlayProxyApp {
    pub(crate) fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state,
        }
    }
}

impl eframe::App for OverlayProxyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Ok(mut state) = self.state.try_write() {
                if state.connected_clients > 0 {
                    ui.add(widgets::StatusIndicator::connected(state.connected_clients.to_string()));
                } else {
                    ui.add(widgets::StatusIndicator::disconnected());
                }
            }
        });
    }
}