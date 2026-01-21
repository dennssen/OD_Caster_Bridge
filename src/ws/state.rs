use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PlayerColor {
    pub primary: Color,
    pub secondary: Color,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Player {
    #[serde(rename = "playerId")]
    pub player_id: u16,
    #[serde(rename = "playerName")]
    pub player_name: String,
    pub root: Transform,
    pub head: Transform,
    #[serde(rename = "leftHand")]
    pub left_hand: Transform,
    #[serde(rename = "rightHand")]
    pub right_hand: Transform,
    pub velocity: Vec3,
    #[serde(rename = "teamIndex")]
    pub team_index: i8,
    #[serde(rename = "playerColor")]
    pub player_color: PlayerColor,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Spectator {
    #[serde(rename = "playerId")]
    pub player_id: u16,
    #[serde(rename = "playerName")]
    pub player_name: String,
    pub ping: u16,
    pub transform: Transform,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Ball {
    pub transform: Transform,
    pub velocity: Vec3,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GameData {
    pub players: Vec<Player>,
    pub spectators: Vec<Spectator>,
    pub balls: Vec<Ball>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SimpleGamemode {
    #[serde(rename = "slotId")]
    pub slot_id: String,
    pub position: Vec3,
    pub rotation: Quat,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SimplePlayer {
    #[serde(rename = "playerId")]
    pub player_id: u16,
    #[serde(rename = "playerName")]
    pub player_name: String,
    position: Vec3,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GamemodeTeam {
    pub score: i8,
    #[serde(rename = "roundsWon")]
    pub rounds_won: i16,
    pub players: Vec<SimplePlayer>,
    #[serde(rename = "teamColor")]
    pub team_color: TeamColor,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct TeamColor {
    pub primary: Color,
    pub secondary: Color,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GamemodeData {
    #[serde(rename = "slotId")]
    pub slot_id: String,
    #[serde(rename = "timeSeconds")]
    pub time_seconds: f32,
    #[serde(rename = "secondaryTimeSeconds")]
    pub secondary_time_seconds: f32,
    #[serde(rename = "isGameRunning")]
    pub is_game_running: bool,
    #[serde(rename = "totalRounds")]
    pub total_rounds: i8,
    #[serde(rename = "useBestOf")]
    pub use_best_of: bool,
    pub teams: Vec<GamemodeTeam>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CasterTeam {
    pub name: String,
    pub logo: String,
}

impl Default for CasterTeam {
    fn default() -> Self {
        Self {
            name: String::default(),
            logo: String::default(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CasterTeams {
    pub home: CasterTeam,
    pub away: CasterTeam,
}

impl Default for CasterTeams {
    fn default() -> Self {
        Self {
            home: CasterTeam::default(),
            away: CasterTeam::default(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GameState {
    #[serde(rename = "gameData")]
    pub game_data: Option<GameData>,
    pub gamemodes: Vec<SimpleGamemode>,
    #[serde(rename = "selectedGamemode")]
    pub selected_gamemode: Option<GamemodeData>,
    pub cameras: Vec<String>,
    #[serde(rename = "selectedCameraConfig")]
    pub selected_camera_config: Option<String>,
    #[serde(rename = "casterTeams")]
    pub caster_teams: CasterTeams,
}