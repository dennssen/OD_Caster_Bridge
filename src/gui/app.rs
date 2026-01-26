use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use eframe::egui;
use indexmap::IndexMap;
use crate::gui::widgets;
use crate::managers::rounds::{RoundManager, RoundOverride};
use crate::ws::state::{CameraApi, GameState, Round};

enum RoundAction {
    Override(Round),
    Delete,
    None
}

pub struct AppState {
    pub game_state: GameState,
    pub subscribed_gamemode_slot_id: String,
    pub camera_api_id: String,

    pub round_manager: RoundManager,

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
    selected_round: usize,
}

impl OverlayProxyApp {
    pub(crate) fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state,
            selected_round: 1,
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

                ui.collapsing("Rounds", |ui| {
                    if let Some(api) = state.game_state.camera_api.as_ref() {
                        let rounds: &IndexMap<usize, Round> = &api.rounds;

                        ui.add(widgets::RoundsPicker::new(rounds, &mut self.selected_round));

                        let round_action: RoundAction = if let Some(round) = rounds.get(&self.selected_round) {
                            let mut round = round.clone();
                            let mut changed: bool = false;
                            let mut should_delete: bool = false;

                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label("Home");
                                    if ui.add(egui::DragValue::new(&mut round.home).suffix(" score")).changed() {
                                        changed = true;
                                    }
                                });

                                ui.add_space(20.0);

                                ui.vertical(|ui| {
                                    ui.label("Away");
                                    if ui.add(egui::DragValue::new(&mut round.away).suffix(" score")).changed() {
                                        changed = true;
                                    }
                                });

                                ui.add_space(20.0);

                                if ui.button("Delete").clicked() {
                                    should_delete = true;
                                }
                            });

                            if should_delete {
                                RoundAction::Delete
                            } else if changed {
                                RoundAction::Override(Round {home: round.home, away: round.away})
                            } else {
                                RoundAction::None
                            }
                        } else {
                            RoundAction::None
                        };

                        match round_action {
                            RoundAction::Override(round) => {
                                state.round_manager.add_override(self.selected_round, RoundOverride::Replace(round));
                            }
                            RoundAction::Delete => {
                                state.round_manager.add_override(self.selected_round, RoundOverride::Delete);
                            }
                            RoundAction::None => {}
                        }
                    }
                });
            });
        });
    }
}