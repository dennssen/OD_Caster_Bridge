use std::sync::Arc;
use tokio::sync::RwLock;
use eframe::egui;
use crate::gui::widgets;
use crate::ws::state::GameState;

pub struct AppState {
    pub game_state: GameState,
    pub subscribed_gamemode_slot_id: String,
    pub subscribed_camera_id: String,

    pub home_team_name: String,
    pub away_team_name: String,
    pub home_team_logo: Option<String>,
    pub away_team_logo: Option<String>,

    pub connected_clients: usize,
    pub spectator_connection: bool,
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
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            let state = self.state.blocking_write();

            if state.connected_clients > 0 && state.spectator_connection {
                ui.add(widgets::StatusIndicator::connected(state.connected_clients.to_string()));
            } else if state.connected_clients == 0 && !state.spectator_connection {
                ui.add(widgets::StatusIndicator::disconnected());
            } else {
                ui.add(widgets::StatusIndicator::connecting());
                ui.label(
                    if state.connected_clients == 0 {
                        "No overlays active."
                    } else {
                        "Missing spectator API. Open the spectator client and join a server."
                    }
                );
            }
        });
    }
}