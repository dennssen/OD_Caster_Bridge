use std::sync::Arc;
use tokio::sync::RwLock;
use eframe::{egui, Frame};
use eframe::egui::{Id, Pos2, RichText, SizeHint, TextureOptions, Ui, ViewportBuilder, ViewportId};
use eframe::egui::load::{TextureLoadResult, TexturePoll};
use indexmap::IndexMap;
use crate::gui::widgets;
use crate::http::logos::{get_and_upload_logo, Team};
use crate::managers::appdata::AppData;
use crate::managers::rounds::{RoundManager, RoundOverride};
use crate::ws::state::{GameState, Round};

enum RoundAction {
    Create,
    Override(usize, Round),
    Delete(usize),
    None
}

pub struct AppState {
    pub game_state: GameState,
    pub subscribed_gamemode_slot_id: String,
    pub camera_api_id: String,

    pub round_manager: RoundManager,
    pub selected_round: Option<usize>,

    pub connected_clients: usize,
    pub spectator_connection: bool,
    pub poll_interval_fps: u64,
    pub poll_game_data: bool,
    pub poll_gamemodes: bool,
    pub poll_cameras: bool,
    pub broadcast_tx: tokio::sync::broadcast::Sender<GameState>,
}

pub struct GUIData {
    state: Arc<RwLock<AppState>>,
    window_position: Option<Pos2>,
    first_frame: bool,
}

impl GUIData {
    pub(crate) fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state,
            window_position: None,
            first_frame: true,
        }
    }
}

impl eframe::App for GUIData {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        // ui.request_repaint();

        egui::CentralPanel::default().show_inside(ui, |ui| {
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

                    ui.scope(|ui| {
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Button,
                            egui::FontId::new(10.0, egui::FontFamily::Proportional)
                        );
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Body,
                            egui::FontId::new(10.0, egui::FontFamily::Proportional)
                        );

                        if ui.button("Settings").clicked() {
                            self.settings = true;
                        }
                    });

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
                                if let Ok(texture) = does_image_exist(ui, &state.game_state.caster_teams.home.logo_url) {
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
                                if let Ok(texture) = does_image_exist(ui, &state.game_state.caster_teams.away.logo_url) {
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
                                let rounds_amount: usize = state.round_manager.get_total_rounds_amount();
                                let rounds: &IndexMap<usize, Round> = &state.round_manager.update_rounds();

                                ui.add(widgets::RoundsPicker::new(rounds, &mut state.selected_round, rounds_amount));

                                let round_action: RoundAction = if let Some(selected_round) = &state.selected_round {
                                    if let Some(round) = rounds.get(selected_round) {
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
                                            RoundAction::Delete(*selected_round)
                                        } else if changed {
                                            RoundAction::Override(*selected_round, Round {home: round.home, away: round.away})
                                        } else {
                                            RoundAction::None
                                        }
                                    } else {
                                        println!("{} | {:?}", selected_round, rounds.keys().collect::<Vec<_>>());
                                        RoundAction::Create
                                    }
                                } else {
                                    RoundAction::None
                                };

                                match round_action {
                                    RoundAction::Create => {
                                        println!("Created new round");
                                        state.round_manager.add_round(Round::default());

                                    }
                                    RoundAction::Override(round_index, round) => {
                                        state.round_manager.add_override(round_index, RoundOverride::Replace(round));
                                    }
                                    RoundAction::Delete(round_index) => {
                                        state.selected_round = None;
                                        state.round_manager.add_override(round_index, RoundOverride::Delete);
                                    }
                                    RoundAction::None => {}
                                }
                            });
                        });
                    });
                });
        });
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        if self.first_frame {
            ctx.set_pixels_per_point(1.5);

            self.first_frame = false;
        }

        let outer_rect = ctx.input(|i| i.viewport().outer_rect);
        if let Some(outer_rect) = outer_rect {
            let native_ppp = ctx.input(|i| i.viewport().native_pixels_per_point.unwrap_or(1.0));
            let position = outer_rect.left_top() * (ctx.pixels_per_point() / native_ppp);
            self.window_position = Some(position);
        }

    }

    fn on_exit(&mut self) {
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

fn does_image_exist(ui: &Ui, source: &str) -> TextureLoadResult {
    ui.try_load_texture(source, TextureOptions::default(), SizeHint::Width(50))
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