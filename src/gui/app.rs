use std::sync::Arc;
use tokio::sync::RwLock;
use eframe::egui;
use eframe::egui::{Pos2, RichText, SizeHint, TextureOptions};
use eframe::egui::load::{TextureLoadResult, TexturePoll};
use eframe::glow::Context;
use indexmap::IndexMap;
use crate::gui::widgets;
use crate::http::logos::{get_and_upload_logo, Team};
use crate::managers::appdata::AppData;
use crate::managers::rounds::{RoundManager, RoundOverride};
use crate::ws::state::{GameState, Round};

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
    window_position: Option<Pos2>,
    first_frame: bool,
}

impl OverlayProxyApp {
    pub(crate) fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state,
            selected_round: 1,
            window_position: None,
            first_frame: true,
        }
    }
}

impl eframe::App for OverlayProxyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.first_frame {
            ctx.set_pixels_per_point(1.5);

            self.first_frame = false;
        }
        ctx.request_repaint();

        let outer_rect = ctx.input(|i| i.viewport().outer_rect);
        if let Some(outer_rect) = outer_rect {
            let position = outer_rect.left_top();
            self.window_position = Some(position);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_viewport(ui, |ui, _| {
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
                        if state.poll_game_data {
                            ui.label(RichText::new("⚠ On high pop (20-30+) this will lag the spectator\nIf you don't know what you are doing keep this off")
                                .small()
                                .color(ui.visuals().error_fg_color));
                        }
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

                    ui.collapsing("Match Data", |ui| {
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
                                "Match Host",
                                &mut state.game_state.match_data.host
                            ).with_desired_width(100.0));
                            ui.add(widgets::TinyTextEdit::single_line(
                                "Match Name",
                                &mut state.game_state.match_data.name
                            ).with_desired_width(100.0));
                            ui.add(widgets::TinyTextEdit::single_line(
                                "Match Stage",
                                &mut state.game_state.match_data.stage
                            ).with_desired_width(100.0));

                            ui.spacing();

                            ui.collapsing("Home Team", |ui| {
                                ui.add(widgets::TinyTextEdit::single_line(
                                    "Team Name",
                                    &mut state.game_state.caster_teams.home.name
                                ).with_desired_width(100.0));
                                ui.horizontal(|ui| {
                                    if ui.button("Image...").clicked() {
                                        get_and_upload_logo(Team::Home(state.game_state.caster_teams.home.name.clone()), &mut state);
                                    }
                                    ui.label("Team Logo");
                                });
                                if let Ok(texture) = does_image_exist(ctx, &state.game_state.caster_teams.home.logo_url) {
                                    if let Some(image_size) = get_image_size(texture) {
                                        ui.horizontal(|ui| {
                                            ui.add(
                                                egui::Image::new(&state.game_state.caster_teams.home.logo_url)
                                                    .fit_to_exact_size(image_size)
                                            );
                                            if ui.button("X").clicked() {
                                                state.game_state.caster_teams.home.logo_url = String::new();
                                            }
                                        });
                                    }
                                }
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
                                        get_and_upload_logo(Team::Away(state.game_state.caster_teams.away.name.clone()), &mut state);
                                    }
                                    ui.label("Team Logo");
                                });
                                if let Ok(texture) = does_image_exist(ctx, &state.game_state.caster_teams.away.logo_url) {
                                    if let Some(image_size) = get_image_size(texture) {
                                        ui.horizontal(|ui| {
                                            ui.add(
                                                egui::Image::new(&state.game_state.caster_teams.away.logo_url)
                                                    .fit_to_exact_size(image_size)
                                            );
                                            if ui.button("X").clicked() {
                                                state.game_state.caster_teams.away.logo_url = String::new();
                                            }
                                        });
                                    }
                                }
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
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&Context>) {
        let state = self.state.blocking_read();
        let window_position: Option<Pos2> = self.window_position;
        let (window_position_x, window_position_y) = match window_position {
            Some(pos) => {
                (Some(pos.x), Some(pos.y))
            }
            None => {
                (None, None)
            }
        };

        let final_app_data: AppData = AppData {
            camera_api_id: state.camera_api_id.clone(),
            poll_interval_fps: state.poll_interval_fps,
            poll_game_data: state.poll_game_data,
            poll_gamemodes: state.poll_gamemodes,
            poll_cameras: state.poll_cameras,
            window_position_x,
            window_position_y
        };

        final_app_data.save();
    }
}

fn does_image_exist(ctx: &egui::Context, source: &str) -> TextureLoadResult {
    ctx.try_load_texture(source, TextureOptions::default(), SizeHint::Width(50))
}

fn get_image_size(texture: TexturePoll) -> Option<egui::Vec2> {
    if let Some(original_size) = texture.size() {
        let aspect_ratio = original_size.y / original_size.x;

        let desired_width = 50.0;
        let calculated_height = desired_width * aspect_ratio;

        Some(egui::vec2(desired_width, calculated_height))
    } else {
        None
    }
}