use std::sync::Arc;
use tokio::sync::RwLock;
use eframe::egui;
use crate::gui::widgets;
use crate::ws::state::GameState;

pub struct AppState {
    pub game_state: GameState,
    pub subscribed_gamemode_slot_id: String,
    pub camera_api_id: String,

    pub connected_clients: usize,
    pub spectator_connection: bool,
    pub poll_interval_fps: u64,
    pub poll_game_data: bool,
    pub poll_gamemodes: bool,
    pub poll_cameras: bool,
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
            let mut state = self.state.blocking_write();

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

            ui.add_space(10.0);
            
            ui.collapsing("Websocket", |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut state.poll_interval_fps, 1..=60));
                    ui.label("Poll Hz");
                });
                
                ui.checkbox(&mut state.poll_game_data, "Poll Game Data");
                ui.checkbox(&mut state.poll_gamemodes, "Poll Gamemodes");
                ui.checkbox(&mut state.poll_cameras, "Poll Cameras");
            });
            
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.scope(|ui| {
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId::new(10.0, egui::FontFamily::Proportional)
                    );
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::new(10.0, egui::FontFamily::Proportional)
                    );

                    ui.add(widgets::TinyTextEdit::single_line(
                        "Camera API",
                        &mut state.camera_api_id
                    ));
                });
            });

            ui.add_space(10.0);

            ui.label("Team Data");

            ui.scope(|ui| {
                ui.style_mut().text_styles.insert(
                    egui::TextStyle::Button,
                    egui::FontId::new(10.0, egui::FontFamily::Proportional)
                );
                ui.style_mut().text_styles.insert(
                    egui::TextStyle::Body,
                    egui::FontId::new(10.0, egui::FontFamily::Proportional)
                );

                ui.collapsing("Home Team", |ui| {
                    ui.add(widgets::TinyTextEdit::single_line(
                        "Team Name",
                        &mut state.game_state.caster_teams.home.name
                    ).with_desired_width(100.0));
                    ui.horizontal(|ui| {
                        if ui.button("Image...").clicked() {
                            todo!()
                        }
                        ui.label("Team Logo");
                    });
                    ui.label("Current Players");
                    ui.add(widgets::PlayerList::list(
                        if let Some(camera_api) = &state.game_state.camera_api {
                            camera_api.home.players.keys().cloned().collect()
                        } else {
                            vec![]
                        }
                    ));
                });

                ui.collapsing("Away Team", |ui| {
                    ui.add(widgets::TinyTextEdit::single_line(
                        "Team Name",
                        &mut state.game_state.caster_teams.away.name
                    ).with_desired_width(100.0));
                    ui.horizontal(|ui| {
                        if ui.button("Image...").clicked() {
                            todo!()
                        }
                        ui.label("Team Logo");
                    });
                    ui.label("Current Players");
                    ui.add(widgets::PlayerList::list(
                        if let Some(camera_api) = &state.game_state.camera_api {
                            camera_api.away.players.keys().cloned().collect()
                        } else {
                            vec![]
                        }
                    ));
                });
            });
        });
    }
}