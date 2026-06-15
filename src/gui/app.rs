use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use eframe::{egui, Frame};
use eframe::egui::{Color32, Id, Modal, Pos2, RichText, SizeHint, TextureOptions, Ui};
use eframe::egui::load::{TextureLoadResult, TexturePoll};
use indexmap::IndexMap;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use regex::Regex;
use crate::gui::widgets;
use crate::http::logos::{get_and_upload_logo, Team};
use crate::managers::appdata::AppData;
use crate::ws::state::{CameraApiUpdate, GameState, Round};

static CLIENT: OnceLock<Client> = OnceLock::new();
static ODC_API_PREFIX: &str = "https://oriondriftcompetitive.com/api/v1";

enum RoundAction {
    Update(),
    Delete(usize),
    None
}

#[derive(PartialEq, PartialOrd)]
pub enum ODCMatchModalState {
    Closed,
    Open,
    Loading,
    Success,
    Error
}

#[derive(Serialize)]
struct UpdatedCameraApi {
    api: CameraApiUpdate
}

#[derive(Deserialize)]
struct MatchJson {
    #[serde(rename = "team1Id")]
    team1_id: String,
    #[serde(rename = "team2Id")]
    team2_id: String,
    #[serde(rename = "weekNumber")]
    week_number: i16,
}

#[derive(Deserialize)]
struct TeamJson {
    name: String,
    #[serde(rename = "profilePicture")]
    profile_picture: String,
}

pub struct AppState {
    pub game_state: GameState,
    pub subscribed_gamemode_slot_id: String,
    pub camera_api_id: String,
    pub odc_match_link: String,

    pub selected_round: Option<usize>,
    pub odc_match_modal_state: ODCMatchModalState,

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
    rt_handle: tokio::runtime::Handle,
}

impl GUIData {
    fn client() -> &'static Client {
        CLIENT.get_or_init(Client::new)
    }

    fn post_updated_rounds(&self, rounds: IndexMap<usize, Round>) {
        let state = self.state.clone();
        self.rt_handle.spawn(async move {
            let client = Self::client();
            let state = state.read().await;

            let json: UpdatedCameraApi = UpdatedCameraApi {
                api: CameraApiUpdate {
                    rounds: Some(rounds),
                    ..Default::default()
                }
            };

            let _ = client
                .post(format!("http://localhost:5420/cameras/{}/config", state.camera_api_id))
                .json(&json)
                .send()
                .await;
        });
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(
        req: reqwest::Result<Response>,
    ) -> Result<T, String> {
        match req {
            Ok(resp) if resp.status().is_success() => {
                resp.json::<T>().await.map_err(|e| e.to_string())
            }
            Ok(resp) => Err(format!("Bad status: {}", resp.status())),
            Err(e) => Err(e.to_string()),
        }
    }

    fn get_odc_match_data(&self) {
        let state = self.state.clone();
        self.rt_handle.spawn(async move {
            let client = Self::client();
            let mut state = state.write().await;
            state.odc_match_modal_state = ODCMatchModalState::Loading;

            let re = Regex::new(r"/matches/(?P<match_id>[a-f0-9]+).*?league=(?P<league_id>[a-f0-9]+)").unwrap();

            let caps = re.captures(state.odc_match_link.as_str());

            if caps.is_none() {
                state.odc_match_modal_state = ODCMatchModalState::Error;
                return;
            }

            let caps = caps.unwrap();
            let match_id = &caps["match_id"];
            let league_id = &caps["league_id"];

            let match_request = client.get(format!("{}/leagues/{}/matches/{}", ODC_API_PREFIX, league_id, match_id)).send().await;

            let match_response: Result<MatchJson, String> = Self::parse_response(match_request).await;

            let (home_team, away_team, week_number) = match match_response {
                Ok(json) => {
                    let home_team_request = client.get(format!("{}/teams/{}", ODC_API_PREFIX, json.team1_id)).send().await;
                    let away_team_request = client.get(format!("{}/teams/{}", ODC_API_PREFIX, json.team2_id)).send().await;

                    let home_team: Result<TeamJson, String> = Self::parse_response(home_team_request).await;
                    let away_team: Result<TeamJson, String> = Self::parse_response(away_team_request).await;

                    (home_team, away_team, json.week_number)
                }
                Err(_) => {
                    state.odc_match_modal_state = ODCMatchModalState::Error;
                    return;
                }
            };

            if home_team.is_err() {
                state.odc_match_modal_state = ODCMatchModalState::Error;
                return;
            }

            if away_team.is_err() {
                state.odc_match_modal_state = ODCMatchModalState::Error;
                return;
            }

            state.game_state.match_data.host = String::from("ODC");
            state.game_state.match_data.stage = format!("Week {}", week_number);

            let (home_team, away_team) = (home_team.unwrap(), away_team.unwrap());

            state.game_state.caster_teams.home.name = home_team.name;
            state.game_state.caster_teams.home.logo_url = home_team.profile_picture;

            state.game_state.caster_teams.away.name = away_team.name;
            state.game_state.caster_teams.away.logo_url = away_team.profile_picture;

            state.odc_match_modal_state = ODCMatchModalState::Success;
        });
    }

    pub(crate) fn new(state: Arc<RwLock<AppState>>, handle: tokio::runtime::Handle) -> Self {
        Self {
            state,
            window_position: None,
            first_frame: true,
            rt_handle: handle
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

                            if state.odc_match_modal_state > ODCMatchModalState::Closed {
                                Modal::new(Id::new("ODC Match Modal"))
                                    .show(ui,|ui| {
                                        ui.label("Get data from ODC match");

                                        ui.add_space(10.0);

                                        ui.add(widgets::TinyTextEdit::single_line(
                                            "Match Link",
                                            &mut state.odc_match_link
                                        ).with_desired_width(100.0));
                                        if state.odc_match_modal_state == ODCMatchModalState::Error {
                                            ui.colored_label(Color32::RED, RichText::new("Bad link").size(10.0));
                                        }

                                        ui.add_space(75.0);

                                        ui.horizontal_centered(|ui| {
                                            if state.odc_match_modal_state == ODCMatchModalState::Loading {
                                                ui.spinner();
                                            } else if ui.button("Get data").clicked() {
                                                self.get_odc_match_data();
                                            }
                                            if ui.button("Cancel").clicked() {
                                                state.odc_match_link = String::new();
                                                state.odc_match_modal_state = ODCMatchModalState::Closed;
                                            }
                                        });

                                        if state.odc_match_modal_state == ODCMatchModalState::Success {
                                            state.odc_match_link = String::new();
                                            state.odc_match_modal_state = ODCMatchModalState::Closed;
                                        }
                                    });
                            }

                            if ui.button("ODC Match").clicked() {
                                state.odc_match_modal_state = ODCMatchModalState::Open;
                            }

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
                                let AppState { game_state, selected_round, ..} = &mut *state;
                                ui.add(widgets::RoundsPicker::new(game_state.camera_api.as_ref().map(|c| &c.rounds), selected_round));

                                if let Some(camera_api) = &mut game_state.camera_api {
                                    let rounds: &mut IndexMap<usize, Round> = &mut camera_api.rounds;

                                    let round_action: Option<RoundAction> = selected_round.and_then(|i| rounds.get_mut(&i).map(|selected_round| {
                                        let mut changed: bool = false;
                                        let mut should_delete: bool = false;

                                        ui.separator();
                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.label("Home");
                                                if ui.add(egui::DragValue::new(&mut selected_round.home).suffix(" score")).changed() {
                                                    changed = true;
                                                }
                                            });

                                            ui.add_space(20.0);

                                            ui.vertical(|ui| {
                                                ui.label("Away");
                                                if ui.add(egui::DragValue::new(&mut selected_round.away).suffix(" score")).changed() {
                                                    changed = true;
                                                }
                                            });

                                            ui.add_space(20.0);

                                            if ui.button("Delete").clicked() {
                                                should_delete = true;
                                            }
                                        });

                                        if changed {
                                            RoundAction::Update()
                                        } else if should_delete {
                                            RoundAction::Delete(i)
                                        } else {
                                            RoundAction::None
                                        }
                                    }));

                                    match round_action {
                                        None => {
                                            if let Some(selected_round_index) = selected_round {
                                                rounds.insert(*selected_round_index, Round::default());
                                                self.post_updated_rounds(rounds.clone());
                                            }
                                        }
                                        Some(action) => {
                                            match action {
                                                RoundAction::Update() => {
                                                    self.post_updated_rounds(rounds.clone());
                                                }
                                                RoundAction::Delete(index) => {
                                                    rounds.shift_remove(&index);

                                                    self.post_updated_rounds(rounds.clone());
                                                    if index > 0 {
                                                        state.selected_round = Some(selected_round.unwrap_or(1) - 1);
                                                    } else {
                                                        state.selected_round = None;
                                                    }
                                                }
                                                RoundAction::None => {}
                                            }
                                        }
                                    }
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